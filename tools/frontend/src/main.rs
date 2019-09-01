mod ast;
mod parse_utils;
mod parser;

use hyper::rt::{self, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};

fn times(msg: &str, d: std::time::Duration) {
    println!(
        "{}: {}",
        msg,
        (d.as_secs() as f32) + (d.as_nanos() as f32) / (1_000_000_000 as f32)
    );
}

fn main() {
    let doc = r##"
   
<!DOCTYPE html>
<html lang="en-US">

<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {
            background-image: linear-gradient(180deg, #47678d, #576b85 35%, #324764 70%, #2a3f5a);
            background-attachment: fixed;
            background-repeat: no-repeat;
            background-position: center top;
            -webkit-background-size: cover;
            -moz-background-size: cover;
            -o-background-size: cover;
            background-size: cover;
            min-height: 100vh;
        }
        
        .no-transitions * {
            transition: none !important;
        }
    </style>
    <script>
        // Random background image
        var bgImages = ["http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/bryggaitonsberg_back3-1920x1280.jpg", "http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/bryggaitonsberg_back4-1920x1280.jpg", "http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/bryggaitonsberg_back5-1920x1280.jpg"];
        var randomBgIdx = Math.floor(Math.random() * Math.floor(bgImages.length));
        var imagePreload = new Image();
        imagePreload.src = bgImages[randomBgIdx];

        function tryBgImage() {
            if (document.body) {
                document.body.style.backgroundImage = 'url(' + bgImages[randomBgIdx] + ')';
            } else {
                window.requestAnimationFrame(tryBgImage);
            }
        }
        imagePreload.onload = function() {
            tryBgImage();
        }
    </script>

    <link rel="pingback" href="http://13.48.23.151/xmlrpc.php" />
    <title>Brygga i Tønsberg</title>
    <link rel='stylesheet' id='front-css' href='http://13.48.23.151/wp-content/themes/brygga-theme/css/front.css' type='text/css' media='all' />
    <link rel='stylesheet' id='brygga_events-css' href='http://13.48.23.151/wp-content/themes/brygga-theme/css/events.css' type='text/css' media='all' />
    <link rel='stylesheet' id='brygga-search-css' href='http://13.48.23.151/wp-content/plugins/brygga/js/search-comp.css?ver=5.2.2' type='text/css' media='all' />
    <link rel='stylesheet' id='opensans-font-css' href='http://13.48.23.151/googlefont/css?family=Arvo:400,700|Open+Sans:400,700&#038;display=swap' type='text/css' media='all' />
    <link rel='stylesheet' id='wp-block-library-css' href='http://13.48.23.151/wp-includes/css/dist/block-library/style.min.css?ver=5.2.2' type='text/css' media='all' />
    <link rel='stylesheet' id='theme-style-css' href='http://13.48.23.151/wp-content/themes/brygga-theme/style.css' type='text/css' media='all' />
    <script type='text/javascript' src='http://13.48.23.151/wp-includes/js/jquery/jquery.js?ver=1.12.4-wp' defer="defer"></script>
    <script type='text/javascript' src='http://13.48.23.151/wp-content/plugins/brygga/js/picturefill.min.js?ver=5.2.2' defer="defer"></script>
    <link rel='https://api.w.org/' href='http://13.48.23.151/wp-json/' />
    <link rel="EditURI" type="application/rsd+xml" title="RSD" href="http://13.48.23.151/xmlrpc.php?rsd" />
    <link rel="wlwmanifest" type="application/wlwmanifest+xml" href="http://13.48.23.151/wp-includes/wlwmanifest.xml" />
    <meta name="generator" content="WordPress 5.2.2" />
</head>

<body class="home blog">
    <div class="brygga-header" role="navigation" aria-label="Hovedmeny">
        <div class="top-menu-left">
            <ul id="menu-hovedmeny" class="brygga-nav">
                <li id="menu-item-17" class="menu-item menu-item-type-custom menu-item-object-custom current-menu-item current_page_item menu-item-17"><a href="/" aria-current="page">Hjem</a></li>
                <li id="menu-item-22" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-22"><a href="http://13.48.23.151/servering-og-uteliv/">Servering og uteliv</a></li>
                <li id="menu-item-18" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-18"><a href="http://13.48.23.151/gjestehavn/">Gjestehavn</a></li>
            </ul>
        </div>
        <div id="brygga-nav-logo">
            <a href="/"><img src="/wp-content/uploads/themes/brygga-theme/img/brygga-i-tonsberg-170x120.png" alt="Brygga i Tønsberg"></a>
        </div>
        <div class="top-menu-right">
            <ul class="brygga-nav">
                <li id="menu-item-20" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-20"><a href="http://13.48.23.151/i-land-i-tonsberg/">I land i Tønsberg</a></li>
                <li id="menu-item-19" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-19"><a href="http://13.48.23.151/historie/">Historie</a></li>
                <li id="menu-item-21" class="menu-item menu-item-type-post_type menu-item-object-page menu-item-21"><a href="http://13.48.23.151/kontakt-oss/">Kontakt oss</a></li>
            </ul>
        </div>
        <!-- end top-menu-right -->
    </div>
    <div id="main-content" class="padding-top">
        <div class="container">

            <div class="brygga-header-content">
                <h1>Velkommen til Brygga i Tønsberg</h1>
                <h2>Norges Beste Brygge!</h2>
            </div>
            <div id="brygga-search-container" role="search">
                <div id="brygga-search-input-wrap">
                    <input type="text" id="brygga-search" placeholder="Sulten? Hva er du fysen på?">
                </div>
            </div>
            <div class="brygga-main-icons" role="navigation" aria-label="Front navigation">
                <a href="/servering-og-uteliv/">
                    <img src="/wp-content/uploads/themes/brygga-theme/img/main-icons/spise-95x95.png" alt="Servering og uteliv">
                    <div class="brygga-main-icons-text">
                        Servering & Uteliv
                    </div>
                </a>
                <a href="#">
                    <img src="/wp-content/uploads/themes/brygga-theme/img/main-icons/gjestehavn-95x95.png" alt="Gjestehavn">
                    <div class="brygga-main-icons-text">
                        Gjestehavn
                    </div>
                </a>
                <a href="#">
                    <img src="/wp-content/uploads/themes/brygga-theme/img/main-icons/opplevelser-95x95.png" alt="Opplevelser">
                    <div class="brygga-main-icons-text">
                        Opplevelser
                    </div>
                </a>
                <a href="#">
                    <img src="/wp-content/uploads/themes/brygga-theme/img/main-icons/overnatting-95x95.png" alt="Overnatting">
                    <div class="brygga-main-icons-text">
                        Overnatting
                    </div>
                </a>
            </div>

            <div class="brygga-events no-transitions" role="list">
                <div id="brygga-events-layout-top">
                    <div class="featured1" role="listitem">
                        <a class="featured1-link" href="https://www.ticketmaster.no/venue/stoperiet-scene-tonsberg-billetter/tsh/3">
                            <img srcset="http://13.48.23.151/wp-content/uploads/2019/07/hellbillies-400x320.jpeg 400w,http://13.48.23.151/wp-content/uploads/2019/07/hellbillies-615x492.jpeg 615w" sizes="(min-width: 1253px) 615px,
            (min-width: 981px) 50vw,
            (max-width: 980px) 80vw" alt="Hellbillies" style="background-image:url(data:image/jpeg;base64,/9j/2wBDAAYEBAUEBAYFBQUGBgYHCQ4JCQgICRINDQoOFRIWFhUSFBQXGiEcFxgfGRQUHScdHyIjJSUlFhwpLCgkKyEkJST/2wBDAQYGBgkICREJCREkGBQYJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCT/wAARCAAUABkDASIAAhEBAxEB/8QAGgAAAgIDAAAAAAAAAAAAAAAAAAcFBgIDBP/EADEQAAIBAgQEBAILAAAAAAAAAAECAwQRAAUSIQYTMUEUIlFhByMyQlNiZIGRkpSh0f/EABcBAAMBAAAAAAAAAAAAAAAAAAECAwD/xAAcEQACAgIDAAAAAAAAAAAAAAAAAQISA0ExMlH/2gAMAwEAAhEDEQA/AEDw9wvLWPSSTyRw08utmLMdShRf6PWx2F/fFpTg+B0oxHyS08azs/iR8lb2721HdenrjTBk3EVLLFKIjIxBTnJ5lAbYhivQ9cd6R5llWV0FQEMjcuOKWKqtYazqTTp+qdPfftgXepFKrwhc6FDAJgqJC9pGmhTSbMh2U269Qfb3xSub95f2nFtejzNswzmWopBUyLKEcBSVLbM1reoA/LEN4CX8J/Ff/MCxnFaG/wANcaVsGTz0dNRZbTrLSlHeODzm67nUSbHvhSVPFObw8s+NkkWncFEkN18pNr+tt7X9cGDEcWxmMv4bV8ua8L5hXVqxzVMtXd5GXdiFAB/vGWiL7CL9MGDCT7MtDg//2Q==)" class="featured1-img brygga-bg-cover">
                            <div class="featured1-top">
                                <div class="featured1-hot">
                                    <div class="featured1-hot-text">BRYGGA&nbsp;ANBEFALER:</div>
                                    <img alt="" src="http://13.48.23.151/wp-content/themes/brygga-theme/img/events/hot-black.png" class="featured1-hot-img"> </div>
                                <div>
                                    <div class="date-box-featured">
                                        <div class="date-num">23.</div>
                                        <div class="date-box-month">Nov.</div>
                                    </div>
                                </div>
                            </div>
                            <img alt="Støperiet" class="featured1-location-img" src="http://13.48.23.151/wp-content/uploads/2019/07/stoperiet.png" />

                            <div class="featured1-text">
                                <h2>Hellbillies</h2>
                                <h3>
                <img alt="Time" draggable="false" class="featured1-icon" src="http://13.48.23.151/wp-content/themes/brygga-theme/img/events/clock.png">
                Lørdag 23. Nov. kl. 20:00 - 23:00<br>
                <img alt="Location" draggable="false" class="featured1-icon" src="http://13.48.23.151/wp-content/themes/brygga-theme/img/events/map-marker.png">
                Støperiet            </h3>
                                <p>
                                    Hellbillies er en rockegruppe fra Ål i Hallingdal som ble startet i 1990. Gruppen har gitt ut elleve studioalbum og fire konsert- og ett samlealbum og de flest har ... </p>
                            </div>
                        </a>
                    </div>
                    <div id="featured2-layout">
                        <div class="featured2" role="listitem">
                            <a class="featured2-link" href="#">
                                <img srcset="http://13.48.23.151/wp-content/uploads/2019/07/bobler-200x133.jpg 200w,http://13.48.23.151/wp-content/uploads/2019/07/bobler-362x241.jpg 362w" sizes="(min-width: 1253px) 615px,
            (min-width: 981px) 30vw,
            (min-width: 768px) 40vw,
            (max-width: 767px) 80vw" alt="Bobler med Dora og Bjørg Thorhallsdot" style="background-image:url(data:image/jpg;base64,/9j/2wBDAAYEBAUEBAYFBQUGBgYHCQ4JCQgICRINDQoOFRIWFhUSFBQXGiEcFxgfGRQUHScdHyIjJSUlFhwpLCgkKyEkJST/2wBDAQYGBgkICREJCREkGBQYJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCT/wAARCAALABADASIAAhEBAxEB/8QAFgABAQEAAAAAAAAAAAAAAAAABgUH/8QAJBAAAQMDBAIDAQAAAAAAAAAAAQIDBBESIQAFBgciMRQVQXH/xAAUAQEAAAAAAAAAAAAAAAAAAAAE/8QAGhEAAgMBAQAAAAAAAAAAAAAAAQIAERIhQf/aAAwDAQACEQMRAD8A2PlHJfopjsGPKHzUtBavELJqMGhwBWmjfXa5r3I5hjOrcjvwCXXZEhTiXJiXB5UuJ9XA20AwNDew2xuMuW9KU465ck3XkEUJFBQ4FPzV/rdRZ2eA8jDiNxS0lXshBFCn+ZONEVyW1fsaUATNdqf/2Q==)" class="featured2-img brygga-bg-cover">
                                <div class="featured2-hot">
                                </div>
                                <div class="featured2-text">
                                    <h2>Bobler med Dora og Bjørg Thorhallsdot</h2>
                                    <h3>
                <img alt="Time" draggable="false" class="featured2-icon" src="http://13.48.23.151/wp-content/themes/brygga-theme/img/events/clock.png">
                Onsdag 20. Nov. kl. 13:00 - 15:30<br>
                <img alt="Location" draggable="false" class="featured2-icon" src="http://13.48.23.151/wp-content/themes/brygga-theme/img/events/map-marker.png">
                Quality Hotel Tønsberg            </h3>
                                </div>
                            </a>
                        </div>
                        <div class="featured2" role="listitem">
                            <a class="featured2-link" href="#">
                                <img srcset="http://13.48.23.151/wp-content/uploads/2019/07/news-200x133.jpg 200w,http://13.48.23.151/wp-content/uploads/2019/07/news-362x241.jpg 362w" sizes="(min-width: 1253px) 615px,
            (min-width: 981px) 30vw,
            (min-width: 768px) 40vw,
            (max-width: 767px) 80vw" alt="Are Kalvø Hyttebok fra helvete" style="background-image:url(data:image/jpg;base64,/9j/2wBDAAYEBAUEBAYFBQUGBgYHCQ4JCQgICRINDQoOFRIWFhUSFBQXGiEcFxgfGRQUHScdHyIjJSUlFhwpLCgkKyEkJST/2wBDAQYGBgkICREJCREkGBQYJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCT/wAARCAALABADASIAAhEBAxEB/8QAFgABAQEAAAAAAAAAAAAAAAAABQIH/8QAIhAAAgEDBQADAQAAAAAAAAAAAQMCBAURAAYSITEyQXGC/8QAFQEBAQAAAAAAAAAAAAAAAAAABAX/xAAeEQABBAEFAAAAAAAAAAAAAAABAAMRIQIEMVGBwf/aAAwDAQACEQMRAD8Azyiv9mp9nWuyPplh1PUtnN64AyAI6Mj6flj+dVS7UUlrZOVBys5A5YBHeDnQNMpTdjVrJqXJka2OGGI5j6wJe47880k+83CntSSqqZA8I9j8xoOobohveYvo+qiy4QRllxIpf//Z)" class="featured2-img brygga-bg-cover">
                                <div class="featured2-hot">
                                </div>
                                <div class="featured2-text">
                                    <h2>Are Kalvø Hyttebok fra helvete</h2>
                                    <h3>
                <img alt="Time" draggable="false" class="featured2-icon" src="http://13.48.23.151/wp-content/themes/brygga-theme/img/events/clock.png">
                Onsdag 20. Nov. kl. 16:30 - 19:30<br>
                <img alt="Location" draggable="false" class="featured2-icon" src="http://13.48.23.151/wp-content/themes/brygga-theme/img/events/map-marker.png">
                Quality Hotel Tønsberg            </h3>
                                </div>
                            </a>
                        </div>
                    </div>
                </div>
                <div class="event-normal-layout">
                    <div class="event-normal-columns">
                        <div class="event-normal-column">
                            <div class="event-normal-container" role="listitem">
                                <div class="event-normal-date-image">
                                    <div class="date-box-normal">
                                        <div class="date-num">15.</div>
                                        <div class="date-box-month">Nov.</div>
                                    </div>
                                    <div class="event-normal-image" style="background-image:url('http://13.48.23.151/wp-content/uploads/2019/07/solveig-sings-138x72.png');"></div>
                                </div>
                                <div class="event-normal-text">
                                    <h2>Jazzkafé med Solveig Sings Jazzkvintett</h2>
                                    <div class="event-normal-clock">
                                        Kl. 20:00 - 21:00 </div>
                                    <div class="event-normal-location">
                                        Quality Hotel Tønsberg </div>
                                </div>
                                <!--
        <div style="float:right">
        </div>-->
                            </div>
                            <div class="event-normal-container" role="listitem">
                                <div class="event-normal-date-image">
                                    <div class="date-box-normal">
                                        <div class="date-num">15.</div>
                                        <div class="date-box-month">Nov.</div>
                                    </div>
                                    <div class="event-normal-image" style="background-image:url('http://13.48.23.151/wp-content/uploads/2019/07/seigmenn-138x72.png');"></div>
                                </div>
                                <div class="event-normal-text">
                                    <h2>Seigmen</h2>
                                    <div class="event-normal-clock">
                                        Kl. 20:00 - 23:00 </div>
                                    <div class="event-normal-location">
                                        Støperiet </div>
                                </div>
                                <!--
        <div style="float:right">
        </div>-->
                            </div>
                            <div class="event-normal-container" role="listitem">
                                <div class="event-normal-date-image">
                                    <div class="date-box-normal">
                                        <div class="date-num">19.</div>
                                        <div class="date-box-month">Nov.</div>
                                    </div>
                                    <div class="event-normal-image" style="background-image:url('http://13.48.23.151/wp-content/uploads/2019/07/quiz-karaoke-138x72.png');"></div>
                                </div>
                                <div class="event-normal-text">
                                    <h2>Quiz & Karaoke Aften</h2>
                                    <div class="event-normal-clock">
                                        Kl. 19:00 - 21:00 </div>
                                    <div class="event-normal-location">
                                        Oseberg Kulturhus </div>
                                </div>
                                <!--
        <div style="float:right">
        </div>-->
                            </div>
                        </div>
                        <div class="event-normal-split"></div>
                        <div class="event-normal-column">
                            <div class="event-normal-container" role="listitem">
                                <div class="event-normal-date-image">
                                    <div class="date-box-normal">
                                        <div class="date-num">22.</div>
                                        <div class="date-box-month">Nov.</div>
                                    </div>
                                    <div class="event-normal-image" style="background-image:url('http://13.48.23.151/wp-content/uploads/2019/07/caledonia-138x72.png');"></div>
                                </div>
                                <div class="event-normal-text">
                                    <h2>Jazzkafé med Caledonia Jazzband</h2>
                                    <div class="event-normal-clock">
                                        Kl. 20:00 - 23:00 </div>
                                    <div class="event-normal-location">
                                        Støperiet </div>
                                </div>
                                <!--
        <div style="float:right">
        </div>-->
                            </div>
                            <div class="event-normal-container" role="listitem">
                                <div class="event-normal-date-image">
                                    <div class="date-box-normal">
                                        <div class="date-num">04.</div>
                                        <div class="date-box-month">Des.</div>
                                    </div>
                                    <div class="event-normal-image" style="background-image:url('http://13.48.23.151/wp-content/uploads/2019/07/punkrock-138x72.png');"></div>
                                </div>
                                <div class="event-normal-text">
                                    <h2>Punkrock live Bagleren Rockpub</h2>
                                    <div class="event-normal-clock">
                                        Kl. 20:00 - 23:00 </div>
                                    <div class="event-normal-location">
                                        Hotell Klubben </div>
                                </div>
                                <!--
        <div style="float:right">
        </div>-->
                            </div>
                            <div class="event-normal-container" role="listitem">
                                <div class="event-normal-date-image">
                                    <div class="date-box-normal">
                                        <div class="date-num">24.</div>
                                        <div class="date-box-month">Des.</div>
                                    </div>
                                    <div class="event-normal-image" style="background-image:url('http://13.48.23.151/wp-content/uploads/2019/07/cake-138x72.jpg');"></div>
                                </div>
                                <div class="event-normal-text">
                                    <h2>Julaften på Harbour</h2>
                                    <div class="event-normal-clock">
                                        Kl. 14:00 - 23:59 </div>
                                    <div class="event-normal-location">
                                        Hotell Klubben </div>
                                </div>
                                <!--
        <div style="float:right">
        </div>-->
                            </div>
                        </div>
                    </div>
                    <div class="event-pager">
                        <div class="event-pager-prev"></div>
                        <div class="event-pager-page  event-pager-current"></div>
                        <div class="event-pager-page "></div>
                        <div class="event-pager-next"></div>
                    </div>
                </div>
            </div>
            <script>
                window.__eventPageData = {
                    "exclude": [79, 77, 75],
                    "date": "2019-07-12 15:58:31",
                    "totalPages": 2,
                    "page1": "<div class=\"event-normal-columns\"><div class=\"event-normal-column\">    <div class=\"event-normal-container\" role=\"listitem\">\n        <div class=\"event-normal-date-image\">\n                <div class=\"date-box-normal\">\n        <div class=\"date-num\">15.<\/div>\n        <div class=\"date-box-month\">Nov.<\/div>\n    <\/div>\n                <div class=\"event-normal-image\" style=\"background-image:url('http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/solveig-sings-138x72.png');\"><\/div>\n        <\/div>\n        <div class=\"event-normal-text\">\n            <h2>Jazzkaf\u00e9 med Solveig Sings Jazzkvintett<\/h2>\n            <div class=\"event-normal-clock\">\n                Kl. 20:00 - 21:00            <\/div>\n            <div class=\"event-normal-location\">\n                Quality Hotel T\u00f8nsberg            <\/div>\n        <\/div>\n        <!--\n        <div style=\"float:right\">\n        <\/div>-->\n    <\/div>\n        <div class=\"event-normal-container\" role=\"listitem\">\n        <div class=\"event-normal-date-image\">\n                <div class=\"date-box-normal\">\n        <div class=\"date-num\">15.<\/div>\n        <div class=\"date-box-month\">Nov.<\/div>\n    <\/div>\n                <div class=\"event-normal-image\" style=\"background-image:url('http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/seigmenn-138x72.png');\"><\/div>\n        <\/div>\n        <div class=\"event-normal-text\">\n            <h2>Seigmen<\/h2>\n            <div class=\"event-normal-clock\">\n                Kl. 20:00 - 23:00            <\/div>\n            <div class=\"event-normal-location\">\n                St\u00f8periet            <\/div>\n        <\/div>\n        <!--\n        <div style=\"float:right\">\n        <\/div>-->\n    <\/div>\n        <div class=\"event-normal-container\" role=\"listitem\">\n        <div class=\"event-normal-date-image\">\n                <div class=\"date-box-normal\">\n        <div class=\"date-num\">19.<\/div>\n        <div class=\"date-box-month\">Nov.<\/div>\n    <\/div>\n                <div class=\"event-normal-image\" style=\"background-image:url('http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/quiz-karaoke-138x72.png');\"><\/div>\n        <\/div>\n        <div class=\"event-normal-text\">\n            <h2>Quiz & Karaoke Aften<\/h2>\n            <div class=\"event-normal-clock\">\n                Kl. 19:00 - 21:00            <\/div>\n            <div class=\"event-normal-location\">\n                Oseberg Kulturhus            <\/div>\n        <\/div>\n        <!--\n        <div style=\"float:right\">\n        <\/div>-->\n    <\/div>\n    <\/div><div class=\"event-normal-split\"><\/div><div class=\"event-normal-column\">    <div class=\"event-normal-container\" role=\"listitem\">\n        <div class=\"event-normal-date-image\">\n                <div class=\"date-box-normal\">\n        <div class=\"date-num\">22.<\/div>\n        <div class=\"date-box-month\">Nov.<\/div>\n    <\/div>\n                <div class=\"event-normal-image\" style=\"background-image:url('http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/caledonia-138x72.png');\"><\/div>\n        <\/div>\n        <div class=\"event-normal-text\">\n            <h2>Jazzkaf\u00e9 med Caledonia Jazzband<\/h2>\n            <div class=\"event-normal-clock\">\n                Kl. 20:00 - 23:00            <\/div>\n            <div class=\"event-normal-location\">\n                St\u00f8periet            <\/div>\n        <\/div>\n        <!--\n        <div style=\"float:right\">\n        <\/div>-->\n    <\/div>\n        <div class=\"event-normal-container\" role=\"listitem\">\n        <div class=\"event-normal-date-image\">\n                <div class=\"date-box-normal\">\n        <div class=\"date-num\">04.<\/div>\n        <div class=\"date-box-month\">Des.<\/div>\n    <\/div>\n                <div class=\"event-normal-image\" style=\"background-image:url('http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/punkrock-138x72.png');\"><\/div>\n        <\/div>\n        <div class=\"event-normal-text\">\n            <h2>Punkrock live Bagleren Rockpub<\/h2>\n            <div class=\"event-normal-clock\">\n                Kl. 20:00 - 23:00            <\/div>\n            <div class=\"event-normal-location\">\n                Hotell Klubben            <\/div>\n        <\/div>\n        <!--\n        <div style=\"float:right\">\n        <\/div>-->\n    <\/div>\n        <div class=\"event-normal-container\" role=\"listitem\">\n        <div class=\"event-normal-date-image\">\n                <div class=\"date-box-normal\">\n        <div class=\"date-num\">24.<\/div>\n        <div class=\"date-box-month\">Des.<\/div>\n    <\/div>\n                <div class=\"event-normal-image\" style=\"background-image:url('http:\/\/13.48.23.151\/wp-content\/uploads\/2019\/07\/cake-138x72.jpg');\"><\/div>\n        <\/div>\n        <div class=\"event-normal-text\">\n            <h2>Julaften p\u00e5 Harbour<\/h2>\n            <div class=\"event-normal-clock\">\n                Kl. 14:00 - 23:59            <\/div>\n            <div class=\"event-normal-location\">\n                Hotell Klubben            <\/div>\n        <\/div>\n        <!--\n        <div style=\"float:right\">\n        <\/div>-->\n    <\/div>\n    <\/div><\/div>"
                };
            </script>
        </div>
        <!-- .container -->
    </div>
    <!-- #main-content -->

    <script>
        document.addEventListener('DOMContentLoaded', (event) => {
            var navLogo = document.getElementById('brygga-nav-logo');
            var isFixed = false;
            var lastScrollTop = 0;
            var hasScrollDown = false;
            var doc = document.documentElement;

            function adjustHeader() {
                var scrollTop = (window.pageYOffset || doc.scrollTop) - (doc.clientTop || 0);
                var direction;
                if (scrollTop === lastScrollTop) {
                    window.requestAnimationFrame(adjustHeader);
                    return;
                } else if (scrollTop > lastScrollTop) {
                    direction = 1;
                } else {
                    direction = -1;
                }
                lastScrollTop = scrollTop;
                // Fix at 97 - 54 = 43
                if (scrollTop >= 43) {
                    if (!isFixed) {
                        document.body.classList.add("brygga-header-fixed");
                        isFixed = true;
                    }
                } else {
                    if (isFixed) {
                        document.body.classList.remove("brygga-header-fixed");
                        isFixed = false;
                    }
                }
                // Set scroll-down or remove
                if (direction === 1 && scrollTop >= 33 && !hasScrollDown) {
                    navLogo.classList.add("brygga-nav-scroll-down");
                    hasScrollDown = true;
                } else if (direction === -1 && scrollTop <= 33 && hasScrollDown) {
                    navLogo.classList.remove("brygga-nav-scroll-down");
                    hasScrollDown = false;
                }
                window.requestAnimationFrame(adjustHeader);
            }
            // Initial call to adjust if we are loading the
            // page already scrolled, then add transitions
            // This will register next requestAnimationFrame
            //adjustHeader();
            setTimeout(() => {
                document.getElementById('brygga-nav-logo').classList.add('brygga-nav-logo-transition');
            }, 50);
        });
    </script>

    <script type='text/javascript'>
        /* <![CDATA[ */
        var brygga_object = {
            "ajax_url": "http:\/\/13.48.23.151\/wp-admin\/admin-ajax.php"
        };
        /* ]]> */
    </script>
    <script type='text/javascript' src='http://13.48.23.151/wp-content/themes/brygga-theme/js/events.js?ver=5.2.2' defer="defer"></script>
    <script type='text/javascript' src='http://13.48.23.151/wp-content/plugins/brygga/js/search-comp.js?ver=5.2.2' defer="defer"></script>
    <div id="modal-layer"></div>
</body>

</html>
    "##;

    let before = std::time::Instant::now();
    let res = parser::parse_doc(doc.as_bytes());
    times("parse", before.elapsed());
    use std::io::Write;
    let mut file = std::fs::File::create("parsed.txt").unwrap();
    write!(file, "{:#?}", res).unwrap();

    pretty_env_logger::init();
    let addr = ([0, 0, 0, 0], 8002).into();
    let server = Server::bind(&addr)
        .serve(|| service_fn_ok(move |_: Request<Body>| Response::new(Body::from("Hello world3"))))
        .map_err(|e| println!("Server error: {:?}", e));
    println!("Listening on: http://{}", addr);
    rt::run(server);
}
