use ratatui::{
    Frame,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
};

use crate::{domain::PassThroughView, repository::WisdomDb};

const SPLASH: &'static str = "
 _______  __   __  _______  ______    _______  __    _
|       ||  | |  ||   _   ||    _ |  |       ||  |  | |
|       ||  |_|  ||  |_|  ||   | ||  |   _   ||   |_| |
|       ||       ||       ||   |_||_ |  | |  ||       |
|      _||       ||       ||    __  ||  |_|  ||  _    |
|     |_ |   _   ||   _   ||   |  | ||       || | |   |
|_______||__| |__||__| |__||___|  |_||_______||_|  |__|



";

// source: https://ascii.co.uk/art
const _CHARON: &'static str = r#"
                                    _._
                                  _/,__\,
                               __/ _/o'o
                             /  '-.___'/  __
                            /__   /\  )__/_))\
 /_/,   __,____             // '-.____|--'  \\
e,e / //  /___/|           |/     \/\        \\
'o /))) : \___\|          /   ,    \/         \\
 -'  \\__,_/|             \/ /      \          \\
         \_\|              \/        \          \\
         | ||              <    '_    \          \\
         | ||             /    ,| /   /           \\
         | ||             |   / |    /\            \\
         | ||              \_/  |   | |             \\
         | ||_______________,'  |__/  \              \\
          \|/_______________\___/______\_             \\
           \________________________     \__           \\        ___
              \________________________    _\_____      \\ _____/
                 \________________________               \\
   ~~~~~~~~  b'ger /  ~~~~~~~~~~~~~~~~~~~~~~~~~~~  ~~ ~~~~\\~~~~
        ~~~~~~~~~~~~~~    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~    //"#;

const CERBERUS: &'static str = r"
            /\_/\____,
  ,___/\_/\ \  ~     /
  \     ~  \ )   XXX
    XXX     /    /\_/\___,
       \o-o/-o-o/   ~    /
        ) /     \    XXX
       _|    / \ \_/
    ,-/   _  \_/   \
   / (   /____,__|  )
  (  |_ (    )  \) _|
 _/ _)   \   \__/   (_
(,-(,(,(,/      \,),),)
";

const GOAT: &'static str = r"
             ,,~~--___---,
            /            .~,
      /  _,~             )
     (_-(~)   ~, ),,,(  /'
      Z6  .~`' ||     \ |
      /_,/     ||      ||
~~~~~~~~~~~~~~~W`~~~~~~W`~~~~~~~~~

";

const BOAT: &'static str = r"
            (\
              \_O
          _____\/)_____
~~~~~~~~~~~`----\----'~~~~~~~~~~~~~~
~~~~~ ~~~~ ~~~~  \ ~~~~~~ ~~~   ~~~~~~

";

pub fn draw_pass_through(f: &mut Frame, view: PassThroughView, db: &WisdomDb) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(get_title(&view));
    let text = get_text(&view, db);
    let vspace = ((f.area().height as usize - text.lines().count()) / 2) - 2;
    let text = format!("{}{}", "\n".repeat(vspace), text);
    let text = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(text, f.area());
}

fn get_title(view: &PassThroughView) -> &'static str {
    pub use PassThroughView::*;
    match view {
        Splash => "",
        Idle => "Idle",
        Speed => "Typing fast!",
        Charonsay => "Charonsay",
    }
}

fn get_text(view: &PassThroughView, db: &WisdomDb) -> String {
    pub use PassThroughView::*;
    let (splash, msg): (&'static str, &str) = match view {
        Splash => (
            SPLASH,
            "Charon is rowing...\n\nPress the <[magic key]> to take control",
        ),
        Idle => (GOAT, db.get_random_wisdom("idle")),
        Speed => (CERBERUS, db.get_random_wisdom("speed")),
        Charonsay => (BOAT, db.get_random_wisdom("charonsay")),
    };
    format!("{}\n\n{}", unify_line_length(splash), msg)
}

fn unify_line_length(txt: &str) -> String {
    let maxlen = txt.lines().map(|line| line.len()).max().unwrap_or(0);
    txt.lines()
        .map(|line| format!("{:<width$}", line, width = maxlen))
        .collect::<Vec<_>>()
        .join("\n")
}
