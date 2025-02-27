#[macro_use] extern crate rocket;

use rocket_dyn_templates::{Template, context};

#[get("/")]
fn index() -> Template {
    let buttons: Vec<i32> = (1..=10).collect(); // Create a vector of pins 1 to 10
    let sliders: Vec<i32> = (11..=13).collect(); // Create a vector of sliders for pins 11, 12, 13
    let context = context! {
        message: "Press the buttons to control GPIO pins!",
        buttons: buttons,
        sliders: sliders, // Add the sliders to the context
    };
    Template::render("index", context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(Template::fairing())
}