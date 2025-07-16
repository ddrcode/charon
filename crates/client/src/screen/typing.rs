use ratatui::{
    Frame,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
};

const SPLASH: &'static str = "


 _______  __   __  _______  ______    _______  __    _
|       ||  | |  ||   _   ||    _ |  |       ||  |  | |
|       ||  |_|  ||  |_|  ||   | ||  |   _   ||   |_| |
|       ||       ||       ||   |_||_ |  | |  ||       |
|      _||       ||       ||    __  ||  |_|  ||  _    |
|     |_ |   _   ||   _   ||   |  | ||       || | |   |
|_______||__| |__||__| |__||___|  |_||_______||_|  |__|




Charon is rowing....

Press the <[magic key]> to take control
";

// source: https://ascii.co.uk/art
const CHARON: &'static str = r#"
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
                 ~~~~~~~~~~~~~~    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~    //
"#;

const CERBERUS: &'static str = r#"
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
         b'ger  (,-(,(,(,/      \,),),)
"#;

pub fn draw_pass_through(f: &mut Frame) {
    let block = Block::default().borders(Borders::ALL);
    let text = Paragraph::new(SPLASH)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(text, f.area());
}
