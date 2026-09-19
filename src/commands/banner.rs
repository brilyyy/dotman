use crate::cli::BannerArgs;
use crate::ui::{self, BannerStyle};
use anyhow::Result;

pub fn execute(args: BannerArgs) -> Result<()> {
    let style = if args.cat {
        BannerStyle::Cat
    } else if args.bot {
        BannerStyle::Bot
    } else if args.font {
        BannerStyle::Font
    } else if args.mini {
        BannerStyle::Mini
    } else {
        BannerStyle::Bunny
    };

    ui::print_banner(style, args.plain);
    Ok(())
}
