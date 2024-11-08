use reporting::{error, note, File, Location, Renderer, Styles};

fn main() {
    let file = File::new("test.txt", "import stds;");
    let styles = Styles::styled();

    print!(
        "{}",
        Renderer::new(
            &styles,
            &[
                error!("Could not find package `{}`", "stds")
                    .location(Location::new(file.clone(), 7)),
                note!("Perhaps you meant `std`?")
            ]
        )
    );
}
