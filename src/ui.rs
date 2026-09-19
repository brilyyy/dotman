use console::{style, StyledObject};
use std::fmt::Display;

pub fn badge_ok() -> StyledObject<&'static str> {
    style("✓").green().bold()
}

pub fn badge_warn() -> StyledObject<&'static str> {
    style("!").yellow().bold()
}

pub fn badge_err() -> StyledObject<&'static str> {
    style("✗").red().bold()
}

pub fn badge_dim() -> StyledObject<&'static str> {
    style("•").dim()
}

pub fn badge_add() -> StyledObject<&'static str> {
    style("+").cyan().bold()
}

pub fn badge_fix() -> StyledObject<&'static str> {
    style("~").magenta().bold()
}

pub fn bold<T: Display>(val: T) -> StyledObject<T> {
    style(val).bold()
}

pub fn dim<T: Display>(val: T) -> StyledObject<T> {
    style(val).dim()
}

pub fn green<T: Display>(val: T) -> StyledObject<T> {
    style(val).green()
}

pub fn yellow<T: Display>(val: T) -> StyledObject<T> {
    style(val).yellow()
}

pub fn red<T: Display>(val: T) -> StyledObject<T> {
    style(val).red()
}

pub fn cyan<T: Display>(val: T) -> StyledObject<T> {
    style(val).cyan()
}

pub fn print_header(title: &str, context: Option<&str>) {
    match context {
        Some(ctx) => println!("{} · {}", style(title).bold(), style(ctx).dim()),
        None => println!("{}", style(title).bold()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BannerStyle {
    #[default]
    Bunny,
    Cat,
    Bot,
    Font,
    Mini,
}

pub fn get_banner_text(style: BannerStyle) -> &'static str {
    match style {
        BannerStyle::Bunny => concat!(
            "       .  ｡ ﾟ ☁︎ ｡ ﾟ  ✧\n",
            "     (\\ (\\   ╭─────────────────────────────╮\n",
            "    ( •.•)  │ · ﾟ ✧  d o t m a n  ✧ ﾟ ·   │\n",
            "    o_(\")(\")│  fast · safe · cozy dotfiles │\n",
            "            ╰─────────────────────────────╯"
        ),
        BannerStyle::Cat => concat!(
            "     |\\__/|,   (`\\\n",
            "   _.|o o  |_   ) )   *  .  ｡  ﾟ  ✧\n",
            " -(((---(((--------  d o t m a n\n",
            "   ╭──────────────╮  [ fast · safe · atomic ]\n",
            "   │ ~/dotfiles/  │  ⋆ ｡ sweet configs tucked in ~\n",
            "   ╰──────────────╯"
        ),
        BannerStyle::Bot => concat!(
            "      ╭─────────────╮\n",
            "      │  (｡♥‿♥｡) ｡ﾟ │   · ﾟ ✧ d o t m a n ✧ ﾟ ·\n",
            "     ╭┤  ── ─┬─ ──  ├╮  [ fast · safe · atomic ]\n",
            "     │╰─────────────╯│  ~ your dotfile companion ~\n",
            "     ╰───┴─┴───┴─┴───╯"
        ),
        BannerStyle::Font => concat!(
            "  . ｡ ˚ ✧                                     ✧ ˚ ｡ .\n",
            "    ╭─╮           _                                  \n",
            "    │ │          | |                                 \n",
            "  ╭─╯ │  ╭───╮ ╭─╯ ├──  ╭─╮╭──╮╭─╮  ╭───╮  ╭────╮    \n",
            "  │ ╭╮│  │ ╭╮│ │ ╭╮│    │ ││ ╭╮│ │  │ ╭╮│  │ ╭╮ │ ｡ ﾟ\n",
            "  │ ╰╯│  │ ╰╯│ │ ╰╯│_   │ ││ │││ │  │ ╭─┤  │ ││ │    \n",
            "  ╰───┴─ ╰───╯ ╰───┴──  ╰─╯╰─╯╰─┴─  ╰───┴─ ╰─╯╰─╯ ✧  \n",
            "            · ﾟ ⋆  f a s t  ·  s a f e  ·  t r u e  ⋆ ﾟ ·"
        ),
        BannerStyle::Mini => concat!(
            "  . ｡ ﾟ ✧\n",
            " (˶ᵔ ᵕ ᵔ˶)  d o t m a n  ·  v1.1.0\n",
            "  / >📦     configs safe & sound ♡"
        ),
    }
}

pub fn print_banner(style: BannerStyle, plain: bool) {
    if plain || !console::colors_enabled() {
        println!("{}", get_banner_text(style));
        return;
    }

    let pink = console::Style::new().color256(218);
    let lavender = console::Style::new().color256(183);
    let cyan = console::Style::new().color256(159);
    let peach = console::Style::new().color256(223);
    let mint = console::Style::new().color256(157);

    match style {
        BannerStyle::Bunny => {
            println!("{}", lavender.apply_to("       .  ｡ ﾟ ☁︎ ｡ ﾟ  ✧"));
            println!(
                "{}   {}",
                pink.apply_to("     (\\ (\\"),
                cyan.apply_to("╭─────────────────────────────╮")
            );
            println!(
                "{}  {} {} {}",
                pink.apply_to("    ( •.•)"),
                cyan.apply_to("│"),
                lavender.apply_to("· ﾟ ✧  d o t m a n  ✧ ﾟ ·"),
                cyan.apply_to("  │")
            );
            println!(
                "{}  {} {} {}",
                pink.apply_to("    o_(\")(\")"),
                cyan.apply_to("│"),
                peach.apply_to(" fast · safe · cozy dotfiles"),
                cyan.apply_to(" │")
            );
            println!("            {}", cyan.apply_to("╰─────────────────────────────╯"));
        }
        BannerStyle::Cat => {
            println!("{}", peach.apply_to("     |\\__/|,   (`\\"));
            println!(
                "{}   {}",
                peach.apply_to("   _.|o o  |_   ) )"),
                lavender.apply_to("*  .  ｡  ﾟ  ✧")
            );
            println!(
                "{}{}",
                peach.apply_to(" -(((---(((--------  "),
                pink.apply_to("d o t m a n")
            );
            println!(
                "{}  {}",
                cyan.apply_to("   ╭──────────────╮"),
                mint.apply_to("[ fast · safe · atomic ]")
            );
            println!(
                "{}  {}",
                cyan.apply_to("   │ ~/dotfiles/  │"),
                peach.apply_to("⋆ ｡ sweet configs tucked in ~")
            );
            println!("{}", cyan.apply_to("   ╰──────────────╯"));
        }
        BannerStyle::Bot => {
            println!("{}", cyan.apply_to("      ╭─────────────╮"));
            println!(
                "{}   {}",
                cyan.apply_to("      │  (｡♥‿♥｡) ｡ﾟ │"),
                lavender.apply_to("· ﾟ ✧ d o t m a n ✧ ﾟ ·")
            );
            println!(
                "{}  {}",
                cyan.apply_to("     ╭┤  ── ─┬─ ──  ├╮"),
                mint.apply_to("[ fast · safe · atomic ]")
            );
            println!(
                "{}  {}",
                cyan.apply_to("     │╰─────────────╯│"),
                peach.apply_to("~ your dotfile companion ~")
            );
            println!("{}", cyan.apply_to("     ╰───┴─┴───┴─┴───╯"));
        }
        BannerStyle::Font => {
            println!("{}", cyan.apply_to("  . ｡ ˚ ✧                                     ✧ ˚ ｡ ."));
            println!("{}", lavender.apply_to("    ╭─╮           _                                  "));
            println!("{}", lavender.apply_to("    │ │          | |                                 "));
            println!("{}", pink.apply_to("  ╭─╯ │  ╭───╮ ╭─╯ ├──  ╭─╮╭──╮╭─╮  ╭───╮  ╭────╮    "));
            println!("{}", pink.apply_to("  │ ╭╮│  │ ╭╮│ │ ╭╮│    │ ││ ╭╮│ │  │ ╭╮│  │ ╭╮ │ ｡ ﾟ"));
            println!("{}", mint.apply_to("  │ ╰╯│  │ ╰╯│ │ ╰╯│_   │ ││ │││ │  │ ╭─┤  │ ││ │    "));
            println!("{}", mint.apply_to("  ╰───┴─ ╰───╯ ╰───┴──  ╰─╯╰─╯╰─┴─  ╰───┴─ ╰─╯╰─╯ ✧  "));
            println!("{}", peach.apply_to("            · ﾟ ⋆  f a s t  ·  s a f e  ·  t r u e  ⋆ ﾟ ·"));
        }
        BannerStyle::Mini => {
            println!("{}", lavender.apply_to("  . ｡ ﾟ ✧"));
            println!(
                "{}  {} {}",
                pink.apply_to(" (˶ᵔ ᵕ ᵔ˶)"),
                cyan.apply_to("d o t m a n"),
                peach.apply_to("·  v1.1.0")
            );
            println!(
                "  {}     {}",
                pink.apply_to("/ >📦"),
                mint.apply_to("configs safe & sound ♡")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banners() {
        assert!(!get_banner_text(BannerStyle::Bunny).is_empty());
        assert!(!get_banner_text(BannerStyle::Cat).is_empty());
        assert!(!get_banner_text(BannerStyle::Bot).is_empty());
        assert!(!get_banner_text(BannerStyle::Font).is_empty());
        assert!(!get_banner_text(BannerStyle::Mini).is_empty());
    }
}

