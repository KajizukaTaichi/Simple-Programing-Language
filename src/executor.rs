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
pub struct Func {
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
    Func,
    Normal,
}

/// コードを実行を管理
pub struct Executor {
    memory: Vec<Variable>, //メモリの変数
    name_space: Vec<Func>, // 関数
    stmt: String,          // ブロックのステートメント
    else_stmt: String,     // elseステートメント
    count: usize,          // ループカウンタ
    name: String,          // 関数の名前
    expr: String,          // 条件式
    mode: Mode,            // 制御ブロックの状態
    old_mode: Mode,        // 元のモード
    nest_if: usize,        // ifネストの階層を表す
    nest_for: usize,       // forネストの階層を表す
    nest_while: usize,     // whileネストの階層を表す
    nest_func: usize,      // funcネストの階層を表す
}

impl Executor {
    /// コンストラクタ
    pub fn new(memory: &Vec<Variable>, name_space: &Vec<Func>) -> Executor {
        Executor {
            memory: memory.to_owned(),
            name_space: name_space.to_owned(),
            stmt: "".to_string(),
            else_stmt: "".to_string(),
            count: 0,
            name: "".to_string(),
            expr: "".to_string(),
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
        let lines = code.trim_start().trim_end();
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
                            println!("{}回目のループ", i + 1);
                            let status = Executor::new(&self.memory, &self.name_space)
                                .execute_block(&self.stmt);
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
                        self.mode = self.old_mode.clone();
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
                        println!("ifの条件式を評価します");
                        if self.calculation(self.expr.clone()) != 0.0 {
                            println!("条件が一致したので、実行します");
                            let status = Executor::new(&self.memory, &self.name_space)
                                .execute_block(&self.stmt);
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
                            println!("条件が一致しなかったので、実行しません");
                            self.stmt = String::new();
                        }
                        self.mode = self.old_mode.clone();
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

            Mode::Func => {
                if lines.contains("end func") {
                    if self.nest_func > 0 {
                        self.nest_func -= 1;
                        self.stmt += lines;
                        self.stmt += "\n";
                    } else {
                        self.name_space.push(Func {
                            name: self.name.clone(),
                            code: self.stmt.clone(),
                        });
                        self.stmt = String::new();
                        self.mode = self.old_mode.clone();
                    }
                } else if lines.contains("func") {
                    self.nest_func += 1;
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
                        println!("ifの条件式を評価します");
                        if self.compute(self.expr.clone()) == 0.0 {
                            println!("条件が一致しなかったので、elseのコードを実行します");
                            let status = Executor::new(&self.memory, &self.name_space)
                                .execute_block(&self.else_stmt);
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
                            println!("条件が一致したので、実行します");
                            let status = Executor::new(&self.memory, &self.name_space)
                                .execute_block(&self.stmt);
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
                        self.mode = self.old_mode.clone();
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
                            println!("whileの条件式を評価します");
                            if self.compute(self.expr.clone()) == 0.0 {
                                self.stmt = String::new();
                                println!("条件が一致しなかったので、ループを脱出します");
                                break;
                            } else {
                                println!("条件が一致したので、ループを継続します");
                                let status = Executor::new(&self.memory, &self.name_space)
                                    .execute_block(&self.stmt);
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
                        self.mode = self.old_mode.clone();
                    }
                } else if lines.contains("while") {
                    self.nest_while += 1;
                } else {
                    self.stmt += lines;
                    self.stmt += "\n";
                }
            }
            _ => {
                if lines.contains("var") {
                    // 変数の定義
                    let new_lines = lines.replacen("var", "", 1);
                    let lines = &new_lines;
                    let params: Vec<&str> = lines.split("=").collect();
                    println!("変数{}を定義しています", params[0].trim().replace(" ", ""));
                    let value = self.compute(params[1..].join("=").to_string());
                    self.memory.push(Variable {
                        name: params[0].trim().replace(" ", ""),
                        value,
                        expr: params[1..].join("=").to_string(),
                    });
                } else if lines.contains("calc") {
                    // 変数の式の再計算
                    let new_lines = lines.replacen("calc", "", 1);
                    let name = &new_lines;
                    match self.reference_variable(name.to_string()) {
                        Some(index) => {
                            let value = self.compute(self.memory[index].to_owned().expr);
                            self.memory[index].value = value;
                            println!("再計算を実行しました");
                        }
                        None => {}
                    }
                } else if lines.contains("func") {
                    //　関数の定義
                    let new_lines = lines.trim().replacen("func", "", 1).replace(" ", "");
                    self.name = new_lines.replace(" ", "");
                    println!("関数{}を定義します", self.name);
                    self.mode = Mode::Func;
                } else if lines.contains("call") {
                    // 関数呼び出し
                    let new_lines = lines.replacen("call", "", 1);
                    let name = &new_lines.replace(" ", "");
                    println!("関数{name}を呼び出します");
                    match self.reference_function(name.clone()) {
                        Some(index) => {
                            Executor::new(&self.memory, &self.name_space)
                                .execute_block(&self.name_space[index].code);
                        }
                        None => {}
                    }
                } else if lines.contains("for") {
                    println!("ループ回数を求めます");
                    let new_lines = lines.replacen("for", "", 1);
                    self.count = self.compute(new_lines) as usize; // ループ回数
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
                    let inputed = input("[入力]> ");
                    let value = self.compute(inputed.clone());
                    self.memory.push(Variable {
                        name: name.to_owned(),
                        value,
                        expr: inputed,
                    });
                } else if lines.contains("print") {
                    //　標準出力
                    let new_lines = lines.replacen("print", "", 1);
                    let mut text = String::new();
                    let params = &new_lines;
                    for i in params.split(",").collect::<Vec<&str>>() {
                        if i.contains("'") || i.contains("\"") {
                            //文字列か？
                            text += &i.replace("'", "").replace("\"", "");
                        } else {
                            //文字列以外は式として扱われる
                            text += self.compute(i.trim().to_string()).to_string().as_str();
                        }
                    }
                    println!("[出力]: {text}");
                } else if lines.find("mem").is_some() {
                    if self.memory.len() != 0 {
                        println!("+-- メモリ内の変数 --");
                        for i in self.memory.iter() {
                            println!(
                                "| name: '{}' - expr: [{}] - value: {}",
                                i.name, i.expr, i.value
                            )
                        }
                    } else {
                        println!("変数がありません");
                    }
                    if self.name_space.len() != 0 {
                        println!("+-- メモリ内の関数 --");
                        for i in self.name_space.iter() {
                            println!("|+-- name: '{}' - len: {}", i.name, i.code.len());
                            let mut number = 0; //行数
                            for j in i.code.split('\n') {
                                if j != "" {
                                    number += 1;
                                    println!("|| [{number}]: {j}");
                                }
                            }
                        }
                    } else {
                        println!("関数がありません");
                    }
                } else if lines.contains("del") {
                    // 変数や関数の削除
                    let new_lines = lines.replacen("del", "", 1);
                    let name = &new_lines;
                    match self.reference_variable(name.clone()) {
                        Some(index) => {
                            self.memory.remove(index);
                            println!("変数{}を削除しました", name);
                        }
                        None => {}
                    }
                    match self.reference_function(name.to_owned()) {
                        Some(index) => {
                            self.memory.remove(index);
                            println!("関数{}を削除しました", name);
                        }
                        None => {}
                    }
                } else if lines.contains("#") { // コメント
                } else if lines.contains("rand") {
                    // 乱数
                    let new_lines = lines.replacen("rand", "", 1);
                    let params = new_lines.split(",").collect::<Vec<&str>>();
                    if params.len() < 3 {
                        let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                        let temp: i64 = rng.gen_range(1, 10);
                        self.memory.push(Variable {
                            name: params[0].trim().replace(" ", ""),
                            value: temp as f64,
                            expr: temp.to_string(),
                        });
                    } else {
                        let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                        let temp: i64 = rng.gen_range(
                            self.compute({
                                println!("最大値を求めます");
                                String::from(params[1])
                            })
                            .round() as i64,
                            self.compute({
                                println!("最小値を求めます");
                                String::from(params[2])
                            })
                            .round() as i64,
                        );
                        self.memory.push(Variable {
                            name: params[0].trim().replace(" ", ""),
                            value: temp as f64,
                            expr: temp.to_string(),
                        });
                    }
                    println!("乱数を生成しました");
                } else if lines.contains("return") {
                    let return_value = lines.replacen("return", "", 1);
                    println!("戻り値を返します");
                    return Some(self.compute(return_value));
                } else if lines.contains("break") {
                    return Some(f64::MAX); //ステータスコード
                } else if lines == "exit" {
                    println!("プロセスを終了します");
                    exit(0);
                } else if lines == "" {
                } else {
                    println!("コマンドが不正です: {}", lines)
                }
                self.remove_duplicates_variable();
                self.remove_duplicates_function();
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
        loop {
            let code = input(">>> ");
            self.execute(code);
        }
    }

    /// ファイルをデバッグする
    pub fn debug(&mut self, code: &String) {
        for lin in code.split("\n") {
            self.execute(lin.to_string());

            // デバッグメニューを表示する
            loop {
                let menu = input("デバッグメニュー>>> ");
                if menu.find("var").is_some() {
                    let lim = &menu.replacen("var", "", 1);
                    let params: Vec<&str> = lim.split("=").collect();
                    let value = self.compute(params[1..].join("=").to_string());

                    self.memory.push(Variable {
                        name: params[0].trim().replace(" ", ""),
                        value,
                        expr: params[1..].join("=").to_string(),
                    });
                } else if menu.find("mem").is_some() {
                    if self.memory.len() != 0 {
                        println!("+-- メモリ内の変数 --");
                        for i in self.memory.iter() {
                            println!(
                                "| name: '{}' - expr: [{}] - value: {}",
                                i.name, i.expr, i.value
                            )
                        }
                    } else {
                        println!("変数がありません");
                    }
                    if self.name_space.len() != 0 {
                        println!("+-- メモリ内の関数 --");
                        for i in self.name_space.iter() {
                            println!("| name: '{}' - len: {}", i.name, i.code.len());
                        }
                    } else {
                        println!("関数がありません");
                    }
                } else if menu.find("exit").is_some() {
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
    fn reference_variable(&self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "");
        match self
            .memory
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => {
                println!("変数{name}が見つかりません");
                None
            }
        }
    }

    /// 関数の参照
    fn reference_function(&self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "");
        match self
            .name_space
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => {
                println!("関数{name}が見つかりません");
                None
            }
        }
    }

    /// 変数の参照(ログ出力なし)
    fn reference_variable_quiet(&self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "");
        self.memory
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
    }

    /// 関数の参照(ログ出力なし)
    fn reference_function_quiet(&self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "");
        self.name_space
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
    }

    /// 変数の重複を削除する
    fn remove_duplicates_variable(&mut self) {
        let mut seen_names = std::collections::HashMap::new();
        let mut to_remove = Vec::new();

        for (index, memory) in self.memory.iter().enumerate() {
            if let Some(existing_index) = seen_names.get(&memory.name) {
                to_remove.push(if existing_index < &index {
                    *existing_index
                } else {
                    index
                });
            } else {
                seen_names.insert(&memory.name, index);
            }
        }

        to_remove.sort(); // Sort indices in ascending order

        for (i, index) in to_remove.iter().enumerate() {
            self.memory.remove(index - i); // Adjust for removed items before
        }
    }

    /// 関数の重複を削除する
    fn remove_duplicates_function(&mut self) {
        let mut seen_names = std::collections::HashMap::new();
        let mut to_remove = Vec::new();

        for (index, memory) in self.memory.iter().enumerate() {
            if let Some(existing_index) = seen_names.get(&memory.name) {
                to_remove.push(if existing_index < &index {
                    *existing_index
                } else {
                    index
                });
            } else {
                seen_names.insert(&memory.name, index);
            }
        }

        to_remove.sort(); // Sort indices in ascending order

        for (i, index) in to_remove.iter().enumerate() {
            self.memory.remove(index - i); // Adjust for removed items before
        }
    }

    /// 式の計算
    fn calculation(&mut self, expr: String) -> f64 {
        let mut stack: Vec<f64> = Vec::new();
        let tokens = expr.split_whitespace();
        for i in tokens {
            let i = i.trim();
            if i.is_empty() {
                continue;
            }
            match i.parse::<f64>() {
                Ok(num) => {
                    stack.push(num);
                    continue;
                }
                Err(_) => {
                    let y = stack.pop().unwrap_or(0.0);
                    let x = stack.pop().unwrap_or(0.0);
                    match i {
                        "+" => stack.push(x + y),
                        "-" => stack.push(x - y),
                        "*" => stack.push(x * y),
                        "/" => stack.push(x / y),
                        "%" => stack.push(x % y),
                        "^" => stack.push(x.powf(y)),
                        "=" => stack.push(if x == y { 1.0 } else { 0.0 }),
                        "&" => stack.push(if x != 0.0 && y != 0.0 { 1.0 } else { 0.0 }),
                        "|" => stack.push(if x != 0.0 || y != 0.0 { 1.0 } else { 0.0 }),
                        ">" => stack.push(if x > y { 1.0 } else { 0.0 }),
                        "<" => stack.push(if x < y { 1.0 } else { 0.0 }),
                        "!" => {
                            stack.push(x);
                            stack.push(if y == 0.0 { 1.0 } else { 0.0 })
                        }
                        _ => {
                            stack.push(x);
                            stack.push(y);

                            match self.reference_variable_quiet(i.to_string()) {
                                Some(i) => {
                                    stack.push(self.memory[i].value);
                                }
                                None => {}
                            }
                            match self.reference_function_quiet(i.to_string()) {
                                Some(i) => stack.push(
                                    Executor::new(&self.memory, &self.name_space)
                                        .execute_block(&self.name_space[i].code)
                                        .unwrap_or(0.0),
                                ),
                                None => {}
                            }
                        }
                    }
                }
            }
        }
        let result = stack.pop().unwrap_or(0.0);
        return result;
    }

    fn compute(&mut self, expr: String) -> f64 {
        let mut stack: Vec<f64> = Vec::new();
        let tokens = expr.split_whitespace();
        println!("+-- 計算処理 --");
        for i in tokens {
            let i = i.trim();
            if i.is_empty() {
                continue;
            }
            println!("| Stack: {:?}  <=  '{}'", stack, i);
            match i.parse::<f64>() {
                Ok(num) => {
                    stack.push(num);
                    continue;
                }
                Err(_) => {
                    let y = stack.pop().unwrap_or(0.0);
                    let x = stack.pop().unwrap_or(0.0);
                    match i {
                        "+" => stack.push(x + y),
                        "-" => stack.push(x - y),
                        "*" => stack.push(x * y),
                        "/" => stack.push(x / y),
                        "%" => stack.push(x % y),
                        "^" => stack.push(x.powf(y)),
                        "=" => stack.push(if x == y { 1.0 } else { 0.0 }),
                        "&" => stack.push(if x != 0.0 && y != 0.0 { 1.0 } else { 0.0 }),
                        "|" => stack.push(if x != 0.0 || y != 0.0 { 1.0 } else { 0.0 }),
                        ">" => stack.push(if x > y { 1.0 } else { 0.0 }),
                        "<" => stack.push(if x < y { 1.0 } else { 0.0 }),
                        "!" => {
                            stack.push(x);
                            stack.push(if y == 0.0 { 1.0 } else { 0.0 })
                        }
                        _ => {
                            stack.push(x);
                            stack.push(y);

                            match self.reference_variable_quiet(i.to_string()) {
                                Some(i) => {
                                    stack.push(self.memory[i].value);
                                }
                                None => {}
                            }
                            match self.reference_function_quiet(i.to_string()) {
                                Some(index) => stack.push({
                                    println!("関数{i}を呼び出します");
                                    match Executor::new(&self.memory, &self.name_space)
                                        .execute_block(&self.name_space[index].code)
                                    {
                                        Some(indes) => indes,
                                        None => 0.0,
                                    }
                                }),
                                None => {}
                            }
                        }
                    }
                }
            };
        }
        let result = stack.pop().unwrap_or(0.0);
        println!("結果 = {}", result);
        return result;
    }
}
