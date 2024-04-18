// P_1_0_01
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
//
/*
* changing colors and size by moving the mouse
*
* MOUSE
* position x          : size
* position y          : color
*
* KEYS
* s                   : save png
*/

use nannou::prelude::*;

fn main() {
    nannou::app(|app| ())
        .simple_window(view)
        .init_fragment_shader::<"noise.wgsl">()
        .size(720, 720)
        .run();
}

fn view(app: &App, _model: &(), entity: Entity) {
    let draw = app.draw();

    let norm_mouse_y = (app.mouse().y / app.window_rect().h()) + 0.5;
    let norm_mouse_x = (app.mouse().x / app.window_rect().w()) + 0.5;
    draw.background().hsl(norm_mouse_y, 1.0, 0.5);

    // This rect is drawn with the "default" material and sets an emissive.
    draw.rect()
        .w_h(app.mouse().x * 2.5, app.mouse().x * 2.5)
        .hsl(1.0 - (norm_mouse_y), 1.0, 0.5)
        .emissive(Color::hsl(1.0 - norm_mouse_x, 1.0, 0.5));

    // This rect is drawn with a different fragment shader, which multiplies
    // its color by a noise value.
    draw.rect()
        .fragment_shader::<"noise.wgsl">()
        .w_h(app.mouse().x * 2.0, app.mouse().x * 2.0)
        .hsl(1.0 - (norm_mouse_y), 1.0, 0.5);
}


