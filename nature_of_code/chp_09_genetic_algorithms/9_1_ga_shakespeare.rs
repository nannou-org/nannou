// The Nature of Code
// Daniel Shiffman
// http://natureofcode.com

// Genetic Algorithm, Evolving Shakespeare

// Demonstration of using a genetic algorithm to perform a search

// setup()
//  # Step 1: The Population
//    # Create an empty population (an array or ArrayList)
//    # Fill it with DNA encoded objects (pick random values to start)

// draw()
//  # Step 1: Selection
//    # Create an empty mating pool (an empty ArrayList)
//    # For every member of the population, evaluate its fitness based on some criteria / function,
//      and add it to the mating pool in a manner consistant with its fitness, i.e. the more fit it
//      is the more times it appears in the mating pool, in order to be more likely picked for reproduction.

//  # Step 2: Reproduction Create a new empty population
//    # Fill the new population by executing the following steps:
//       1. Pick two "parent" objects from the mating pool.
//       2. Crossover -- create a "child" object by mating these two parents.
//       3. Mutation -- mutate the child's DNA based on a given probability.
//       4. Add the child object to the new population.
//    # Replace the old population with the new population
//
//   # Rinse and repeat
// Example 9-1: GA Shakespeare

use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

// A type to describe a psuedo-DNA, i.e. genotype
//   Here, a virtual organism's DNA is an array of character.
//   Functionality:
//      -- convert DNA into a string
//      -- calculate DNA's "fitness"
//      -- mate DNA with another set of DNA
//      -- mutate DNA

#[derive(Clone)]
struct Dna {
    genes: Vec<char>, // The genetic sequence
    fitness: f32,
}

impl Dna {
    fn new(num: usize) -> Self {
        let mut genes = Vec::new();
        for _ in 0..num {
            genes.push(random_ascii());
        }
        Dna {
            genes,
            fitness: 0.0,
        }
    }

    fn get_phrase(&self) -> String {
        self.genes.iter().cloned().collect()
    }

    // Fitness function (returns floating point % of "correct" characters)
    fn calculate_fitness(&mut self, target: &String) {
        let mut score = 1;
        for i in 0..self.genes.len() {
            if self.genes[i] == target.chars().nth(i).unwrap() {
                score += 1;
            }
        }

        self.fitness = score as f32 / target.len() as f32
    }

    fn crossover(&self, partner: &Dna) -> Dna {
        // A new child
        let mut child = Dna::new(self.genes.len());
        let midpoint = random_range(0, self.genes.len()); // Pick a midpoint

        // Half from one, half from the other
        for i in 0..self.genes.len() {
            if i > midpoint {
                child.genes[i] = self.genes[i];
            } else {
                child.genes[i] = partner.genes[i];
            }
        }
        child
    }

    // Based on a mutation probability, picks a new random character
    fn mutate(&mut self, mutation_rate: f32) {
        for i in 0..self.genes.len() {
            if random_f32() < mutation_rate {
                self.genes[i] = random_ascii();
            }
        }
    }
}

// A type to describe a population of virtual organisms
// In this case, each organism is just an instance of a DNA object
struct Population {
    mutation_rate: f32,    // Mutation rate
    population: Vec<Dna>,  // Vector to hold the current population
    mating_pool: Vec<Dna>, // Vector which we will use for our "mating pool"
    target: String,        // Target phrase
    generations: u32,      // Number of generations
    finished: bool,        // Are we finished evolving?
    perfect_score: f32,
}

impl Population {
    fn new(p: &String, m: f32, num: usize) -> Self {
        let target = p;
        Population {
            mutation_rate: m,
            population: vec![Dna::new(target.len()); num],
            mating_pool: Vec::new(),
            target: target.to_string(),
            generations: 0,
            finished: false,
            perfect_score: 1.0,
        }
    }

    fn calculate_fitness(&mut self) {
        for i in 0..self.population.len() {
            self.population[i].calculate_fitness(&self.target);
        }
    }

    // Generate a mating pool
    fn natural_selection(&mut self) {
        // Clear the vector
        self.mating_pool.clear();
        let mut max_fitness = 0.0;
        for i in 0..self.population.len() {
            if self.population[i].fitness > max_fitness {
                max_fitness = self.population[i].fitness;
            }
        }

        // Based on fitness, each member will get added to the mating pool a certain number of times
        // a higher fitness = more entries to mating pool = more likely to be picked as a parent
        // a lower fitness = fewer entries to mating pool = less likely to be picked as a parent
        for i in 0..self.population.len() {
            let fitness = map_range(self.population[i].fitness, 0.0, max_fitness, 0.0, 1.0);
            let n = fitness as usize * 100; // Arbitrary multiplier, we can also use monte carlo method
            for _ in 0..n {
                self.mating_pool.push(self.population[i].clone());
            }
        }
    }

    // Create a new generation
    fn generate(&mut self) {
        // Refill the population with children from the mating pool
        for i in 0..self.population.len() {
            let a = random_range(0, self.mating_pool.len());
            let b = random_range(0, self.mating_pool.len());
            let partner_a = &self.mating_pool[a];
            let partner_b = &self.mating_pool[b];
            let mut child = partner_a.crossover(partner_b);
            child.mutate(self.mutation_rate);
            self.population[i] = child;
        }
        self.generations += 1;
    }

    // Compute the current "most fit" member of the population
    fn get_best(&mut self) -> String {
        let mut world_record = 0.0;
        let mut index = 0;
        for i in 0..self.population.len() {
            if self.population[i].fitness > world_record {
                index = i;
                world_record = self.population[i].fitness;
            }
        }

        if world_record > self.perfect_score {
            self.finished = true;
        }
        self.population[index].get_phrase()
    }

    // Compute average fitness for the population
    fn get_average_fitness(&self) -> f32 {
        let mut total = 0.0;
        for p in self.population.iter() {
            total += p.fitness;
        }
        total / self.population.len() as f32
    }

    fn all_phrases(&self) -> String {
        let mut everything = "".to_string();
        let display_limit = self.population.len().min(50);
        for i in 0..display_limit {
            everything = format!("{}{} \n", everything, self.population[i].get_phrase());
        }
        everything
    }
}

struct Model {
    answer: String,
    pop_max: usize,
    mutation_rate: f32,
    population: Population,
}

fn model(app: &App) -> Model {
    app.new_window().size(640, 360).view(view).build().unwrap();
    let target = "To be or not to be.".to_string();
    let pop_max = 150;
    let mutation_rate = 0.01;
    // Create a populationation with a target phrase, mutation rate, and populationation max
    let mut population = Population::new(&target, mutation_rate, pop_max);
    population.calculate_fitness();
    Model {
        answer: "".to_string(),
        pop_max,
        mutation_rate,
        population,
    }
}

fn update(_app: &App, model: &mut Model) {
    // Generate mating pool
    model.population.natural_selection();
    // Create next generation
    model.population.generate();
    // Calculate fitness
    model.population.calculate_fitness();
    // Get current status of populationation
    model.answer = model.population.get_best();
}

fn view(app: &App, model: &Model) {
    draw.background().color(WHITE);

    let win = app.window_rect();
    let draw = app.draw();

    draw.text(&"Best Phrase:".to_string())
        .color(BLACK)
        .left_justify()
        .align_text_top()
        .font_size(24)
        .x(20.0)
        .y(-30.0)
        .wh(win.wh());
    draw.text(&model.answer)
        .color(BLACK)
        .left_justify()
        .align_text_top()
        .font_size(40)
        .x(20.0)
        .y(-100.0)
        .wh(win.wh());

    let gen = format!("total generations:     {}", model.population.generations);
    draw.text(&gen)
        .color(BLACK)
        .left_justify()
        .align_text_top()
        .font_size(18)
        .x(20.0)
        .y(-160.0)
        .wh(win.wh());
    let fitness = format!(
        "average fitness:         {:.2}",
        model.population.get_average_fitness()
    );
    draw.text(&fitness)
        .color(BLACK)
        .left_justify()
        .align_text_top()
        .font_size(18)
        .x(20.0)
        .y(-180.0)
        .wh(win.wh());
    let pop = format!("total population:       {}", model.pop_max);
    draw.text(&pop)
        .color(BLACK)
        .left_justify()
        .align_text_top()
        .font_size(18)
        .x(20.0)
        .y(-200.0)
        .wh(win.wh());
    let rate = format!("mutation rate:            {:.2} %", model.mutation_rate);
    draw.text(&rate)
        .color(BLACK)
        .left_justify()
        .align_text_top()
        .font_size(18)
        .x(20.0)
        .y(-220.0)
        .wh(win.wh());

    let all = format!("All phrases: \n {}", model.population.all_phrases());
    draw.text(&all)
        .color(BLACK)
        .left_justify()
        .align_text_top()
        .font_size(10)
        .x(500.0)
        .y(-10.0)
        .wh(win.wh());




    if model.population.finished {
        app.set_update_mode(UpdateMode::freeze());
    }
}
