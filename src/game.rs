use super::*;
pub mod actions;
pub mod helper;
pub struct Game {
    pub stdin: std::io::StdinLock<'static>,
    pub stdout: std::io::StdoutLock<'static>,
    pub rng: rand::rngs::ThreadRng,
    pub time: Time,
    pub locations: Locations,
    pub connections: Vec<Connection>,
    pub player: Player,
}

impl Game {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();
        let mut g: Game = Game {
            stdin: std::io::stdin().lock(),
            stdout: std::io::stdout().lock(),
            rng: rand::rng(),
            time: Time::new(),
            locations: Locations::new(),
            connections: vec![
                Connection {
                    a: Location::StrandedShip,
                    b: Location::Beach,
                    time_a_to_b: Time::s(30),
                    time_b_to_a: Time::s(40),
                },
                Connection {
                    a: Location::Beach,
                    b: Location::Forest,
                    time_a_to_b: Time::m(3),
                    time_b_to_a: Time::m(3),
                },
            ],
            player: Player::new(),
        };
        execute!(g.stdout, EnterAlternateScreen).unwrap();
        g
    }

    pub fn tick_ms(&mut self) {
        use Attribute::*;
        self.time += Time(1);

        self.player.tick();

        if self.player.attrs[Energy] <= Time(0) {
            self.end_game("你被饿死了！");
        }
        if self.player.attrs[Water] <= Time(0) {
            self.end_game("你被渴死了！");
        }
        if self.player.attrs[Health] <= Time(0) {
            self.end_game("你伤重而死！");
        }

        for ld in &mut self.locations.0 {
            ld.tick(self.time, &mut self.rng);
        }
    }

    pub fn end_game(&mut self, message: &str) {
        execute!(self.stdout, MoveTo(0, 0)).unwrap();
        write_lines(&format!("游戏结束！\n{message}"));
        sleep(Duration::from_secs(3));

        execute!(self.stdout, LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        print!("游戏结束：{message}\n");
        std::process::exit(0);
    }

    pub fn run(&mut self) {
        loop {
            execute!(self.stdout, MoveTo(0, 0)).unwrap();
            write_lines("=== On Island ===\n");
            for attr in Attribute::iter() {
                write_lines(&format!("{}: {}\n", attr.name(), self.player.attrs[attr]));
            }
            write_lines(&format!(
                "位置: {}\n[t]旅行 [g]采集 [u]使用物品 [r]休息 [q]退出",
                self.player.location.name()
            ));
            execute!(self.stdout, Clear(FromCursorDown)).unwrap();
            match read().unwrap() {
            Event::Key(key_event) if let KeyCode::Char(ch) = key_event.code => match ch {
            't' => self.travel(),
            'g' => self.gather(),
            'u' => self.use_item(),
            'r' => self.rest(),
            'q' => self.end_game("你关闭了游戏。"),
            _ => {}
            },
            _ => {}
            }
        }
    }
}
