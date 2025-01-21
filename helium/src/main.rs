use helium_renderer;
use pollster::block_on;

fn main() {
    pretty_env_logger::init();
    block_on(helium_renderer::run());
}
