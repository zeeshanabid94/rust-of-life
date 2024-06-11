use cursive::{views::TextView, Cursive, CursiveExt};

fn main() {
    let mut cursive_ref = Cursive::new();

    cursive_ref.add_layer(TextView::new("Hello World!\nPress q to quit."));

    cursive_ref.add_global_callback('q', |cur_ref| cur_ref.quit());

    cursive_ref.run();
}

