
struct Console {
    fn log(a: T) {
        js! {
            console.log(a);
        }
    }
}


console.log("Abc");