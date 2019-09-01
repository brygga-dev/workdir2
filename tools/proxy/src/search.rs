use actix_web::{
    client::Client,
    web::{self, BytesMut, HttpRequest, HttpResponse},
    Error,
};
//use futures::Stream;
use crate::AppData;
use futures::{stream::Stream, Future};
use serde::{Deserialize, Serialize};
use tantivy::schema;

// Menu data
#[derive(Debug, Deserialize)]
struct AllMenus {
    menus: Vec<MenuEntry>,
}
#[derive(Debug, Deserialize)]
struct MenuEntry {
    location_id: u32,
    location_name: String,
    menu: Vec<MenuCategory>,
}
#[derive(Debug, Deserialize)]
struct MenuCategory {
    id: String,
    name: String,
    dishes: Vec<DishEntry>,
}
#[derive(Debug, Deserialize)]
struct DishEntry {
    id: String,
    name: String,
    description: String,
    price: String,
}

/// Holds data related to an index
/// needed in appdata to query and index new
#[derive(Clone)]
struct IndexData {
    index: tantivy::Index,
    reader: tantivy::IndexReader,
}
impl IndexData {
    fn new(index: tantivy::Index, reader: tantivy::IndexReader) -> Self {
        IndexData { index, reader }
    }
}

#[derive(Clone)]
pub struct AllIndexData {
    menu_schema: MenuFields,
    menu_index_data: IndexData,
}

pub fn initial_index_data() -> AllIndexData {
    let menu_schema = menu_schema();
    let menu_index = tantivy::Index::create_in_ram(menu_schema.schema.clone());
    // One reader per index
    let menu_reader = menu_index.reader().unwrap();
    AllIndexData {
        menu_schema,
        menu_index_data: IndexData::new(menu_index, menu_reader),
    }
}

#[derive(Deserialize)]
struct SearchQueryParams {
    s: String,
}
#[derive(Serialize)]
struct SearchResult {
    items: Vec<SearchResultItem>,
}
#[derive(Serialize)]
enum SearchResultItem {
    MenuItem(String),
}

pub fn search_handler(data: web::Data<AppData>, req: HttpRequest) -> HttpResponse {
    let params = match web::Query::<SearchQueryParams>::from_query(&req.query_string()) {
        Ok(params) => params.into_inner(),
        Err(_) => {
            return HttpResponse::Ok().json("Search string `s` required.");
        }
    };
    let result = query_menu_data(&data.index_data, params.s);
    HttpResponse::Ok().json(result)
}

pub fn index_menus(data: web::Data<AppData>) -> impl Future<Item = (), Error = Error> {
    let client = Client::default();
    client
        .get("http://192.168.33.10/wp-json/brygga/all-menus")
        .timeout(std::time::Duration::new(15, 0))
        .send()
        .map_err(Error::from)
        .and_then(move |resp| {
            println!("Got index response: {}", resp.status().as_u16());
            let status_code = resp.status().as_u16();
            println!("Status code: {}", status_code);
            resp.from_err()
                .fold(BytesMut::new(), |mut acc, chunk| {
                    acc.extend_from_slice(&chunk);
                    Ok::<_, Error>(acc)
                })
                .map(move |body| {
                    let body: AllMenus = serde_json::from_slice(&body).unwrap();
                    index_menu_data(body, &data.index_data);
                    // One reader per index
                    ()
                })
        })
}

fn index_menu_data(all_menus: AllMenus, index_data: &AllIndexData) {
    // This is 10 mb ram
    let mut index_writer = index_data.menu_index_data.index.writer(10_000_000).unwrap();
    for menu in all_menus.menus {
        for category in menu.menu {
            for dish in category.dishes {
                index_writer.add_document(doc!(
                    index_data.menu_schema.location_id => menu.location_id as u64,
                    index_data.menu_schema.location_name => menu.location_name.clone(),
                    index_data.menu_schema.category_name => category.name.clone(),
                    index_data.menu_schema.dish_name => dish.name.clone(),
                    index_data.menu_schema.dish_description => dish.description.clone()
                ));
            }
        }
    }
    index_writer.commit().unwrap();
}

fn query_menu_data(index_data: &AllIndexData, query: String) -> SearchResult {
    let searcher = index_data.menu_index_data.reader.searcher();
    let query_parser = tantivy::query::QueryParser::for_index(
        &index_data.menu_index_data.index,
        vec![
            index_data.menu_schema.location_name,
            index_data.menu_schema.category_name,
            index_data.menu_schema.dish_name,
            index_data.menu_schema.dish_description,
        ],
    );
    let query = query_parser.parse_query(&query).unwrap();
    println!("{:?}", query);
    let mut collector = tantivy::collector::TopDocs::with_limit(10);
    let top_docs = searcher.search(&*query, &mut collector).unwrap();
    let items = top_docs
        .into_iter()
        .map(|(_score, doc_address)| {
            let doc = searcher.doc(doc_address).unwrap();
            SearchResultItem::MenuItem(
                doc.get_first(index_data.menu_schema.dish_name)
                    .unwrap()
                    .text()
                    .unwrap()
                    .into(),
            )
            //println!("{}", index_data.menu_schema.schema.to_json(&doc));
        })
        .collect();
    SearchResult { items }
}

#[derive(Clone)]
pub struct MenuFields {
    location_id: schema::Field,
    location_name: schema::Field,
    category_name: schema::Field,
    dish_name: schema::Field,
    dish_description: schema::Field,
    schema: schema::Schema,
}

pub fn menu_schema() -> MenuFields {
    use schema::*;
    let mut schema_builder = SchemaBuilder::default();
    // location_id
    let location_id_opts = IntOptions::default().set_stored().set_indexed();
    let location_id = schema_builder.add_u64_field("location_id", location_id_opts);
    // location_name
    let location_name = schema_builder.add_text_field("location_name", TEXT | STORED);
    // category_name
    let category_name = schema_builder.add_text_field("category_name", TEXT | STORED);
    // dish_name
    let dish_name = schema_builder.add_text_field("dish_name", TEXT | STORED);
    // dish_description
    let dish_description = schema_builder.add_text_field("dish_description", TEXT | STORED);

    let schema = schema_builder.build();

    MenuFields {
        location_id,
        location_name,
        category_name,
        dish_name,
        dish_description,
        schema,
    }
}
