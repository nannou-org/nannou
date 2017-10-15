fn main() {
    App::run::<Model>();
}


#[derive(Model)]
struct Model {
    // State that lives on the main thread.
    #[app(main)]
    main: main::Model,
    // State that lives on the audio thread.
    #[app(audio)]
    audio: audio::Model,
}

fn model(app: &App) -> Model {
    unimplemented!()
}


////////////////
///// MAIN /////
////////////////


mod main {
    struct Model {}

    // NOTE: Could possibly provide a couple default `Event` types that implement `MainEvent`
    // (Event60Fps, EventWait, etc).
    #[derive(MainEvent)]
    enum Event {
        // `WindowEvent`s describe user interactions with a particular window.
        //
        // Each `WindowEvent` contains a `WindowId` indicating which window was interacted with.
        WindowEvent(WindowEvent),
    
        // `Timer` events will be emitted
        //
        // A "minimum"
        // A "catch-up" timer will attempt to catch up to where it was if it falls behind.
        #[event(timer = "catch-up")]
        Update(Timer),
    
        // Users can also listen for custom event types.
        //
        // Custom events must be registered with some Listener<Custom> that listens on a
        // mpsc::Receiver<Custom>.
        #[event(custom = )]
        Custom(Custom)
    }

    fn update(app: &App, event: Event, model: Model) -> Model {
        match event {
            Event::WindowEvent(_event) => {
            },
            Event::Update(_timer) => {
            },
            Event::Custom(_custom) => {
            },
        }
        main
    }

    fn view(app: &App, model: &Model) -> VisualFrame {
        unimplemented!()
    }
}


/////////////////
///// Audio /////
/////////////////


mod audio {
    struct Model {}

    #[derive(AudioEvent)]
    enum Event {
        SampleRate(SampleRate),
        Channels(Channels),
    }

    fn update(app: &App, event: Event, model: Model) -> Model {
        match event {
            AudioEvent::SampleRate(_hz) => {
            },
            AudioEvent::Channels(_channels) => {
            }
        }
        audio
    }

    fn hear(app: &App, model: &Model) -> AudioFrame {
        unimplemented!()
    }
}
