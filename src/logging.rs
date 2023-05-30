use tracing_subscriber::{filter::FilterFn, layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn init() {
	std::env::set_var("RUST_BACKTRACE", "1");

	log::set_max_level(log::LevelFilter::Info);
	log::set_boxed_logger(Box::<tinylog::Logger>::default()).expect("impossible");

	tracing_subscriber::registry()
		.with(
			tinylog::Logger::default().with_filter(FilterFn::new(|meta| {
				if *meta.level() == tracing::Level::DEBUG {
					match meta
						.module_path()
						.unwrap_or_else(|| meta.target())
						.split(':')
						.next()
					{
						Some(name) => name == env!("CARGO_PKG_NAME"),
						None => false,
					}
				} else {
					true
				}
			})),
		)
		.try_init()
		.expect("impossible");
}
