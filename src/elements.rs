type ReRunTimer = Option<Duration>;

enum Text {
    Static(String),
    Command(String, ReRunTimer),
}

struct Slider {
    min: f64,
    max: f64,
    value: f64,
    step: f64,
    on_change_command: String
}