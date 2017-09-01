
fn main() {
    App::run::<Model>();
}


#[derive(Model)]
struct Model {
    // State that lives on the main thread.
    #[app(main)]
    main: Main,
    // State that lives on the audio thread.
    #[app(audio)]
    audio: Audio,
}

fn model(app: &App) -> Model {
}


////////////////
///// MAIN /////
////////////////


struct Main {}

#[derive(MainEvent)]
enum MainEvent {
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


fn update_main(app: &App, event: MainEvent, main: Main) -> Main {
    match event {
        MainEvent::WindowEvent(_event) => {
        },
        MainEvent::Update(_timer) => {
        },
        MainEvent::Custom(_custom) => {
        },
    }
    main
}

fn view(app: &App, main: &Main) -> VisualFrame {
    unimplemented!();
}


/////////////////
///// Audio /////
/////////////////


struct Audio {}

#[derive(AudioEvent)]
enum AudioEvent {
    SampleRate(SampleRate),
    Channels(Channels),
}

fn update_audio(app: &App, event: AudioEvent, audio: Audio) -> Audio {
    audio
}

fn hear(app: &App, audio: &Audio) -> AudioFrame {
    unimplemented!();
}
