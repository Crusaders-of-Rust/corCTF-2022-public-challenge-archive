mod slice;
mod context;
mod machine;

use machine::Machine;

fn main() {
    Machine::load("./machine.dat").run();
}
