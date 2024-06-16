// M_6_2_01
//
// Generative Gestaltung – Creative Coding im Web
// ISBN: 978-3-87439-902-9, First Edition, Hermann Schmidt, Mainz, 2018
// Benedikt Groß, Hartmut Bohnacker, Julia Laub, Claudius Lazzeroni
// with contributions by Joey Lee and Niels Poldervaart
// Copyright 2018
//
// http://www.generative-gestaltung.de
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/**
 * loads the names of the links on the wikipedia-site "Superegg"
 * and prints them to the console
 */
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model;

fn model(app: &App) -> Model {
    app.new_window().size(400, 400).view(view).build();

    let wiki = wikipedia::Wikipedia {
        client: wikipedia::http::default::Client::default(),
        links_results: "500".to_string(),
        ..Default::default()
    };

    let page = wiki.page_from_title("Superegg".to_owned());
    let links = page.get_links().unwrap();

    links.enumerate().for_each(|(i, link)| {
        println!("Link {}: {} ", i, link.title);
    });

    Model
}

fn update(_app: &App, _model: &mut Model) {}

fn view(app: &App, _model: &Model) {
    // Begin drawing
    let draw = app.draw();
}
