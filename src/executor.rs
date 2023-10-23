use rand::Rng;
use std::io::{self, Write};
use std::process::exit;

/// 標準入力を取得する
fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    return result
        .trim_start()
        .trim_end()
        .parse()
        .ok()
        .unwrap_or("".to_string());
}

/// 変数のデータ
#[derive(Clone)]
pub struct Variable {
    name: String,
    value: f64,
    expr: String,
}

/// 関数のデータ
#[derive(Clone)]
pub struct Function {
    name: String,
    code: String,
}

/// 制御モード
#[derive(Clone)]
enum Mode {
    If,
    Else,
    For,
    While,
    Function,
    Normal,
}

/// コードを実行を管理
pub struct Executor {
    memory: Vec<Variable>,     //　メモリ内の変数
    name_space: Vec<Function>, // 関数の名前空間
    stmt: String,              // ブロックのステートメント
    else_stmt: String,         // elseステートメント
    count: usize,              // ループカウンタ
    name: String,              // 関数の名前
    expr: String,              // 条件式
    log: bool,                 // ログ出力
    mode: Mode,                // 制御ブロックの状態
    old_mode: Mode,            // 元のモード
    nest_if: usize,            // ifネストの階層を表す
    nest_for: usize,           // forネストの階層を表す
    nest_while: usize,         // whileネストの階層を表す
    nest_func: usize,          // funcネストの階層を表す
}

impl Executor {
    /// コンストラクタ
    pub fn new(memory: &mut Vec<Variable>, name_space: &mut Vec<Function>, log: bool) -> Executor {
        Executor {
            memory: memory.to_owned(),
            name_space: name_space.to_owned(),
            stmt: "".to_string(),
            else_stmt: "".to_string(),
            count: 0,
            name: "".to_string(),
            expr: "".to_string(),
            log,
            mode: Mode::Normal,
            old_mode: Mode::Normal,
            nest_if: 0,
            nest_for: 0,
            nest_while: 0,
            nest_func: 0,
        }
    }

    /// 文の実行
    pub fn execute(&mut self, code: String) -> Option<f64> {
        let lines = code.trim().split("#").collect::<Vec<&str>>()[0];
        match self.mode {
            Mode::For => {
                if lines.contains("end for") {
                    // ネストの階層を判別する
                    if self.nest_for > 0 {
                        self.nest_for -= 1;
                        self.stmt += lines;
                        self.stmt += "\n";
                    } else {
                        for i in 0..self.count {
                            if self.log {
                                println!("{}回目のループ", i + 1);
                            }
                            let status =
                                Executor::new(&mut self.memory, &mut self.name_space, self.log)
                                    .execute_block(&mut self.stmt);
                            match status {
                                Some(i) => {
                                    if i == f64::MAX {
                                        //状態が1(break)の時はループを抜け出す
                                        break;
                                    } else {
                                        // 戻り値を返す
                                        return Some(i);
                                    }
                                }
                                None => {}
                            }
                        } // モードを元に戻す
                        self.stmt = String::new();
                        self.mode = Mode::Normal;
                    }
                } else if lines.contains("for") {
                    // ネストの階層を上げる
                    self.nest_for += 1;
                    self.stmt += lines;
                    self.stmt += "\n";
                } else {
                    // コードを追加する
                    self.stmt += lines;
                    self.stmt += "\n";
                }
            }

            Mode::If => {
                if lines.contains("else") {
                    // モードをelseに変える
                    self.old_mode = self.mode.clone();
                    self.mode = Mode::Else
                } else if lines.contains("end if") {
                    if self.nest_if > 0 {
                        self.nest_if -= 1;
                        self.stmt += lines;
                        self.stmt += "\n";
                    } else {
                        // 条件式を評価する
                        if self.log {
                            println!("ifの条件式を評価します");
                        }
                        if self.compute(self.expr.clone()) != 0.0 {
                            if self.log {
                                println!("条件が一致したので、実行します");
                            }
                            let status =
                                Executor::new(&mut self.memory, &mut self.name_space, self.log)
                                    .execute_block(&mut self.stmt);
                            match status {
                                Some(i) => {
                                    if i == f64::MAX {
                                        // ループ階層へ渡す
                                        return Some(f64::MAX);
                                    } else {
                                        // 戻り値を返す
                                        return Some(i);
                                    }
                                }
                                None => {}
                            }
                            self.stmt = String::new();
                        } else {
                            if self.log {
                                println!("条件が一致しなかったので、実行しません");
                            }
                            self.stmt = String::new();
                        }
                        self.mode = Mode::Normal;
                    }
                } else if lines.contains("if") {
                    self.nest_if += 1;
                    self.stmt += lines;
                    self.stmt += "\n";
                } else {
                    self.stmt += lines;
                    self.stmt += "\n";
                }
            }

            Mode::Else => {
                if lines.contains("end if") {
                    if self.nest_if > 0 {
                        self.nest_if -= 1;
                        self.stmt += lines;
                        self.stmt += "\n";
                    } else {
                        if self.log {
                            println!("ifの条件式を評価します");
                        }
                        if self.compute(self.expr.clone()) == 0.0 {
                            if self.log {
                                println!("条件が一致しなかったので、elseのコードを実行します");
                            }
                            let status =
                                Executor::new(&mut self.memory, &mut self.name_space, self.log)
                                    .execute_block(&mut self.else_stmt);
                            match status {
                                Some(i) => {
                                    if i == f64::MAX {
                                        return Some(f64::MAX);
                                    } else {
                                        return Some(i);
                                    }
                                }
                                None => {}
                            }
                            self.else_stmt = String::new();
                            self.stmt = String::new();
                        } else {
                            if self.log {
                                println!("条件が一致したので、実行します");
                            }
                            let status =
                                Executor::new(&mut self.memory, &mut self.name_space, self.log)
                                    .execute_block(&mut self.stmt);
                            match status {
                                Some(i) => {
                                    if i == f64::MAX {
                                        return Some(f64::MAX);
                                    } else {
                                        return Some(i);
                                    }
                                }
                                None => {}
                            }
                            self.else_stmt = String::new();
                            self.stmt = String::new();
                        }
                        self.mode = Mode::Normal;
                    }
                } else if lines.contains("if") {
                    self.nest_if += 1;
                    self.stmt += lines;
                    self.stmt += "\n";
                    self.mode = Mode::If;
                } else {
                    self.else_stmt += lines;
                    self.else_stmt += &String::from("\n");
                }
            }
            Mode::While => {
                if lines.contains("end while") {
                    if self.nest_while > 0 {
                        self.nest_while -= 1;
                    } else {
                        loop {
                            if self.log {
                                println!("whileの条件式を評価します");
                            }
                            if self.compute(self.expr.trim().to_string()) == 0.0 {
                                self.stmt = String::new();
                                if self.log {
                                    println!("条件が一致しなかったので、ループを脱出します");
                                }
                                break;
                            } else {
                                if self.log {
                                    println!("条件が一致したので、ループを継続します");
                                }
                                let status =
                                    Executor::new(&mut self.memory, &mut self.name_space, self.log)
                                        .execute_block(&mut self.stmt);
                                match status {
                                    Some(i) => {
                                        if i == f64::MAX {
                                            //状態が1(break)の時はループを抜け出す
                                            break;
                                        } else {
                                            return Some(i);
                                        }
                                    }
                                    None => {}
                                }
                            }
                        }
                        self.mode = Mode::Normal;
                    }
                } else if lines.contains("while") {
                    self.nest_while += 1;
                } else {
                    self.stmt += lines;
                    self.stmt += "\n";
                }
            }

            Mode::Function => {
                if lines.contains("end func") {
                    if self.nest_func > 0 {
                        self.nest_func -= 1;
                        self.stmt += lines;
                        self.stmt += "\n";
                    } else {
                        self.set_function(self.name.clone(), self.stmt.clone());
                        self.stmt = String::new();
                        self.mode = Mode::Normal;
                    }
                } else if lines.contains("func") {
                    self.nest_func += 1;
                } else {
                    self.stmt += lines;
                    self.stmt += "\n";
                }
            }

            Mode::Normal => {
                if lines.contains("var") {
                    // 変数の定義
                    let new_lines = lines.replacen("var", "", 1);
                    let lines = &new_lines;
                    let params: Vec<&str> = lines.split("=").collect();
                    if self.log {
                        println!("変数{}を定義しています", params[0].trim().replace(" ", ""));
                    }

                    self.set_variable(
                        params[0].trim().replace(" ", ""),
                        params[1..].join("=").to_string(),
                    );
                } else if lines.contains("calc") {
                    // 変数の式の再計算
                    let new_lines = lines.replacen("calc", "", 1);
                    let name = &new_lines;
                    let expr = self.get_variable_expr(name.to_string());
                    self.set_variable(name.to_string(), expr);
                    if self.log {
                        println!("再計算を実行しました");
                    }
                } else if lines.contains("func") {
                    //　関数の定義
                    let new_lines = lines.trim().replacen("func", "", 1).replace(" ", "");
                    self.name = new_lines.replace(" ", "").replace("(", "").replace(")", "");
                    if self.log {
                        println!("関数{}を定義します", self.name);
                    }
                    self.mode = Mode::Function;
                } else if lines.contains("call") {
                    // 関数呼び出し
                    self.call_function(lines.replacen("call", "", 1));
                } else if lines.contains("for") {
                    if self.log {
                        println!("ループ回数を求めます");
                    }
                    let new_lines = lines.replacen("for", "", 1);
                    self.count = self.compute(new_lines).round() as usize; // ループ回数
                    self.old_mode = self.mode.clone();
                    self.mode = Mode::For;
                } else if lines.contains("if") {
                    let new_lines = lines.replacen("if", "", 1);
                    self.expr = new_lines;
                    self.old_mode = self.mode.clone();
                    self.mode = Mode::If
                } else if lines.contains("while") {
                    let new_lines = lines.replacen("while", "", 1);
                    self.expr = new_lines;
                    self.old_mode = self.mode.clone();
                    self.mode = Mode::While;
                } else if lines.contains("input") {
                    // 標準入力
                    let new_lines = lines.replacen("input", "", 1);
                    let name = &new_lines;
                    let inputed = if self.log {
                        println!("標準入力を受け取ります");
                        input("[入力]> ")
                    } else {
                        input("> ")
                    };
                    self.set_variable(name.trim().replace(" ", ""), inputed.to_string());
                } else if lines.contains("print") {
                    //　標準出力
                    let new_lines = lines.replacen("print", "", 1);
                    let mut text = String::new();
                    let params = &new_lines;
                    if self.log {
                        println!("標準出力に表示します");
                    }
                    for i in params.split(",").collect::<Vec<&str>>() {
                        if i.contains("'") || i.contains("\"") {
                            //文字列か？
                            text += &i.replace("'", "").replace("\"", "");
                        } else {
                            //文字列以外は式として扱われる
                            text += self.compute(i.trim().to_string()).to_string().as_str();
                        }
                    }
                    if self.log {
                        println!("[出力]: {text}");
                    } else {
                        println!("{text}");
                    }
                } else if lines.contains("mem") {
                    if self.memory.len() != 0 {
                        if self.log {
                            println!("+-- メモリ内の変数 --");
                        }
                        for i in self.memory.iter() {
                            println!(
                                "| name: '{}' - expr: [{}] - value: {}",
                                i.name, i.expr, i.value
                            )
                        }
                    } else {
                        if self.log {
                            println!("変数がありません");
                        }
                    }
                    if self.name_space.len() != 0 {
                        if self.log {
                            println!("+-- メモリ内の関数 --");
                        }
                        for i in self.name_space.iter() {
                            if self.log {
                                println!("|+-- name: '{}' - len: {}", i.name, i.code.len());
                            }
                            let mut number = 0; //行数
                            for j in i.code.split('\n') {
                                if j != "" {
                                    number += 1;
                                    if self.log {
                                        println!("|| [{number}]: {j}");
                                    }
                                }
                            }
                        }
                    } else {
                        if self.log {
                            println!("関数がありません");
                        }
                    }
                } else if lines.contains("del") {
                    // 変数や関数の削除
                    let new_lines = lines.replacen("del", "", 1);
                    let name = &new_lines;
                    match self.reference_variable(name.clone()) {
                        Some(index) => {
                            self.memory.remove(index);
                            if self.log {
                                println!("変数{}を削除しました", name);
                            }
                        }
                        None => {}
                    }
                    match self.reference_function(name.to_owned()) {
                        Some(index) => {
                            self.name_space.remove(index);
                            if self.log {
                                println!("関数{}を削除しました", name);
                            }
                        }
                        None => {}
                    }
                } else if lines.contains("rand") {
                    // 乱数
                    let new_lines = lines.replacen("rand", "", 1);
                    let params = new_lines.split(",").collect::<Vec<&str>>();
                    if params.len() < 3 {
                        let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                        let temp: i64 = rng.gen_range(1, 10);
                        self.set_variable(params[0].trim().replace(" ", ""), temp.to_string());
                    } else {
                        let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                        let temp: i64 = rng.gen_range(
                            self.compute({
                                if self.log {
                                    println!("最小値を求めます");
                                }
                                String::from(params[1])
                            })
                            .round() as i64,
                            self.compute({
                                if self.log {
                                    println!("最大値を求めます");
                                }
                                String::from(params[2])
                            })
                            .round() as i64,
                        );
                        self.set_variable(params[0].trim().replace(" ", ""), temp.to_string());
                    }
                    if self.log {
                        println!("乱数を生成しました");
                    }
                } else if lines.contains("return") {
                    let return_value = lines.replacen("return", "", 1);
                    if self.log {
                        println!("戻り値を返します");
                    }
                    return Some(self.compute(return_value));
                } else if lines.contains("break") {
                    if self.log {
                        println!("ループを脱出します");
                    }
                    return Some(f64::MAX); //ステータスコード
                } else if lines == "exit" {
                    if self.log {
                        println!("プロセスを終了します");
                    }
                    exit(0);
                } else if lines == "" {
                } else {
                    if self.log {
                        println!("コマンドが不正です: {}", lines)
                    }
                }
            }
        }
        return None;
    }

    /// ブロックを実行
    pub fn execute_block(&mut self, code: &String) -> Option<f64> {
        for lin in code.split("\n") {
            if let Some(i) = self.execute(lin.to_string()) {
                // 戻り値を返す
                return Some(i);
            }
        }
        return None;
    }

    /// REPLで対話的に実行する
    pub fn interactive(&mut self) {
        self.log = true;
        loop {
            let code = input(
                format!(
                    "{}> ",
                    match self.mode {
                        Mode::If => "If分岐",
                        Mode::Else => "Else分岐",
                        Mode::For => "Forループ",
                        Mode::While => "Whileループ",
                        Mode::Normal => "プログラム",
                        Mode::Function => "関数定義",
                    }
                )
                .as_str(),
            );
            self.execute(code);
        }
    }

    /// スクリプトを実行する
    pub fn script(&mut self, code: &String) {
        self.log = false;
        self.execute_block(code);
    }

    /// ファイルをデバッグする
    pub fn debug(&mut self, code: &String) {
        self.log = true;
        for lin in code.split("\n") {
            self.execute(lin.to_string());

            // デバッグメニューを表示する
            loop {
                let menu = input("デバッグメニュー>>> ");
                if menu.contains("var") {
                    let lim = &menu.replacen("var", "", 1);
                    let params: Vec<&str> = lim.split("=").collect();
                    let value = self.compute(params[1..].join("=").to_string());

                    self.memory.push(Variable {
                        name: params[0].trim().replace(" ", ""),
                        value,
                        expr: params[1..].join("=").to_string(),
                    });
                } else if menu.contains("mem") {
                    if self.memory.len() != 0 {
                        if self.log {
                            println!("+-- メモリ内の変数 --");
                        }
                        for i in self.memory.iter() {
                            println!(
                                "| name: '{}' - expr: [{}] - value: {}",
                                i.name, i.expr, i.value
                            )
                        }
                    } else {
                        if self.log {
                            println!("変数がありません");
                        }
                    }
                    if self.name_space.len() != 0 {
                        if self.log {
                            println!("+-- メモリ内の関数 --");
                        }
                        for i in self.name_space.iter() {
                            if self.log {
                                println!("| name: '{}' - len: {}", i.name, i.code.len());
                            }
                        }
                    } else {
                        if self.log {
                            println!("関数がありません");
                        }
                    }
                } else if menu.contains("exit") {
                    input("デバッグを中断します");
                    exit(0);
                } else {
                    input("継続します");
                    break;
                }
            }
        }
    }

    /// 変数の参照
    fn reference_variable(&mut self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "").replace("　", "");
        match self
            .memory
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => {
                if self.log {
                    println!("変数{name}が見つかりません");
                }
                None
            }
        }
    }

    ///　関数を呼び出す
    fn call_function(&mut self, name: String) -> f64 {
        let name = name
            .replace(" ", "")
            .replace("　", "")
            .replace("(", "")
            .replace(")", "");
        let code = match self.get_function(name.clone()) {
            Some(func) => func.code,
            None => return 0.0,
        };
        if self.log {
            println!("関数{name}を呼び出します");
        }
        match Executor::new(&mut self.memory, &mut self.name_space, self.log).execute_block(&code) {
            Some(indes) => indes,
            None => 0.0,
        }
    }

    /// 関数を定義する
    fn set_function(&mut self, name: String, code: String) {
        let name = name.trim().replace(" ", "").replace("　", "");
        let is_duplicate = {
            //関数が既に在るか？
            let mut flag = false;
            for item in self.name_space.iter() {
                if item.name == name.trim().replace(" ", "") {
                    flag = true;
                }
            }
            flag
        };

        if is_duplicate {
            //　関数が在る場合は更新する
            let address = self.reference_function(name).unwrap_or(0);
            self.name_space[address].code = code.clone();
        } else {
            //ない場合は新規に確保する
            self.name_space.push(Function { name, code });
        }
    }

    /// 関数の参照
    fn reference_function(&mut self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "").replace("　", "");
        match self
            .name_space
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => {
                if self.log {
                    println!("関数{name}が見つかりません");
                }
                None
            }
        }
    }

    /// 変数の値をセットする
    fn set_variable(&mut self, name: String, expr: String) {
        let name = name.trim().replace(" ", "").replace("　", "");
        let is_duplicate = {
            //変数が既に在るか？
            let mut flag = false;
            for item in self.memory.iter() {
                if item.name == name.trim().replace(" ", "") {
                    flag = true;
                }
            }
            flag
        };

        if is_duplicate {
            //　変数が在る場合は更新する
            let address = self.reference_variable(name).unwrap_or(0);
            self.memory[address].expr = expr.clone();
            self.memory[address].value = self.compute(expr.clone());
        } else {
            //ない場合は新規に確保する
            let value = self.compute(expr.clone());
            self.memory.push(Variable { name, value, expr });
        }
    }

    /// 変数を取得する
    fn get_variable(&mut self, name: String) -> Option<Variable> {
        let name = name.trim().replace(" ", "").replace("　", "");
        let index = match self.reference_variable(name) {
            Some(i) => i,
            None => return None,
        };
        return Some(self.memory[index].clone());
    }

    /// 変数の値を取得する
    fn get_variable_value(&mut self, name: String) -> f64 {
        if self.log {
            println!("変数{name}を参照します");
        }
        match self.get_variable(name) {
            Some(i) => i.value,
            None => return 0.0,
        }
    }

    /// 変数の式を取得する
    fn get_variable_expr(&mut self, name: String) -> String {
        if self.log {
            println!("変数{name}を参照します");
        }
        match self.get_variable(name) {
            Some(i) => i.expr,
            None => return "".to_string(),
        }
    }

    /// 関数を取得する
    fn get_function(&mut self, name: String) -> Option<Function> {
        let index = match self.reference_function(name) {
            Some(i) => i,
            None => return None,
        };
        return Some(self.name_space[index].clone());
    }

    /// 式の計算
    fn compute(&mut self, expr: String) -> f64 {
        let mut stack: Vec<f64> = Vec::new();
        let tokens = expr.split_whitespace();
        if self.log {
            println!("+-- 計算処理 --");
        }
        for item in tokens {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            if self.log {
                println!("| Stack: {:?}  <=  '{}'", stack, item);
            }
            match item.parse::<f64>() {
                Ok(number) => {
                    stack.push(number);
                    continue;
                }
                Err(_) => {
                    let y = stack.pop().unwrap_or(0.0);
                    let x = stack.pop().unwrap_or(0.0);
                    match item {
                        "+" | "＋" => stack.push(x + y),
                        "-" | "ー" => stack.push(x - y),
                        "*" | "＊" | "×" => stack.push(x * y),
                        "/" | "÷" => stack.push(x / y),
                        "%" | "％" => stack.push(x % y),
                        "^" | "＾" => stack.push(x.powf(y)),
                        "=" | "＝" => stack.push(if x == y { 1.0 } else { 0.0 }),
                        "&" | "＆" => stack.push(if x != 0.0 && y != 0.0 { 1.0 } else { 0.0 }),
                        "|" | "｜" => stack.push(if x != 0.0 || y != 0.0 { 1.0 } else { 0.0 }),
                        ">" | "＞" => stack.push(if x > y { 1.0 } else { 0.0 }),
                        "<" | "＜" => stack.push(if x < y { 1.0 } else { 0.0 }),
                        "!" | "！" => {
                            stack.push(x);
                            stack.push(if y == 0.0 { 1.0 } else { 0.0 })
                        }
                        _ => {
                            stack.push(x);
                            stack.push(y);

                            if item.contains("(") || item.contains(")") {
                                stack.push(self.call_function(item.to_string()))
                            } else {
                                stack.push(self.get_variable_value(item.to_string()));
                            }
                        }
                    }
                }
            };
        }
        let result = stack.pop().unwrap_or(0.0);
        if self.log {
            println!("結果 = {}", result);
        }
        return result;
    }
}
