use wizardy_derive::Wizard;

#[derive(Debug, Wizard)]
pub struct Foo {
    pub color: String,
    pub quest: String,
    pub speed: String,
}

fn main() {
    let foo = Foo::wizard()
        .ask_color("What is your favorite color ?")
        .ask_quest("What is your quest?")
        .ask_speed("What Is the Airspeed Velocity of an Unladen Swallow?")
        .run();

    println!("{:?}", foo);
}
