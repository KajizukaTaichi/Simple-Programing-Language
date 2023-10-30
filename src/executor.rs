use rand::Rng;
use std::io::{self, Write};
use std::process::exit;

/// 標準入力を取得する
fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    return result.trim().parse().ok().unwrap_or("".to_string());
}

/// 変数のデータ
#[derive(Clone)]
pub struct Variable {
    name: String,
    value: f64,
}

/// 関数のデータ
#[derive(Clone)]
pub struct Function {
    name: String,
    args: Vec<String>,
    code: Vec<String>,
}

/// 制御モード
#[derive(Clone)]
enum ControlMode {
    If,
    Else,
    For,
    While,
    Function,
    Normal,
}

/// 実行モード
#[derive(Clone)]
pub enum ExecutionMode {
    Interactive,
    Script,
    Debug,
}

/// コードを実行を管理
pub struct Executor<'a> {
    memory: &'a mut Vec<Variable>,     //　メモリ内の変数
    name_space: &'a mut Vec<Function>, // 関数の名前空間
    stmt: Vec<String>,                 // ブロックのステートメント
    else_stmt: Vec<String>,            // elseステートメント
    count: usize,                      // ループカウンタ
    data: String,                      // 関数のデータ
    expr: String,                      // 条件式
    control_mode: ControlMode,         // 制御ブロックの状態
    execution_mode: ExecutionMode,     // 制御ブロックの状態
    nest_if: usize,                    // ifネストの階層を表す
    nest_for: usize,                   // forネストの階層を表す
    nest_while: usize,                 // whileネストの階層を表す
    nest_func: usize,                  // funcネストの階層を表す
}

impl<'a> Executor<'a> {
    /// コンストラクタ
    pub fn new(
        memory: &'a mut Vec<Variable>,
        name_space: &'a mut Vec<Function>,
        mode: ExecutionMode,
    ) -> Executor<'a> {
        Executor {
            memory: memory,
            name_space: name_space,
            stmt: Vec::new(),
            else_stmt: Vec::new(),
            count: 0,
            data: "".to_string(),
            expr: "".to_string(),
            control_mode: ControlMode::Normal,
            execution_mode: mode,
            nest_if: 0,
            nest_for: 0,
            nest_while: 0,
            nest_func: 0,
        }
    }

    /// 文の実行
    pub fn execute(&mut self, code: String) -> Option<f64> {
        match self.control_mode {
            ControlMode::For => {
                if code.contains("end for") {
                    // ネストの階層を判別する
                    if self.nest_for > 0 {
                        self.nest_for -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        for i in 0..self.count {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("{}回目のループ", i + 1);
                            }
                            if let ExecutionMode::Interactive = self.execution_mode {
                                self.execution_mode = ExecutionMode::Debug
                            }
                            let status = Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .execute_block(self.stmt.clone());
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
                        self.stmt = Vec::new();
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("for") {
                    // ネストの階層を上げる
                    self.nest_for += 1;
                    self.stmt.push(code.to_string());
                } else {
                    // コードを追加する
                    self.stmt.push(code.to_string());
                }
            }

            ControlMode::If => {
                if code.contains("else") {
                    // モードをelseに変える
                    self.control_mode = ControlMode::Else
                } else if code.contains("end if") {
                    if self.nest_if > 0 {
                        self.nest_if -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("ifの条件式を評価します");
                        }
                        if self.compute(self.expr.clone()) != 0.0 {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("条件が一致したので、実行します");
                            }
                            if let ExecutionMode::Interactive = self.execution_mode {
                                self.execution_mode = ExecutionMode::Debug
                            }
                            let status = Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .execute_block(self.stmt.clone());
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
                            self.stmt = Vec::new();
                        } else {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("条件が一致しなかったので、実行しません");
                            }
                            self.stmt = Vec::new();
                        }
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("if") {
                    self.nest_if += 1;
                    self.stmt.push(code.to_string());
                } else {
                    self.stmt.push(code.to_string());
                }
            }

            ControlMode::Else => {
                if code.contains("end if") {
                    if self.nest_if > 0 {
                        self.nest_if -= 1;
                        self.else_stmt.push(code.to_string());
                    } else {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("ifの条件式を評価します");
                        }
                        if self.compute(self.expr.clone()) == 0.0 {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("条件が一致しなかったので、elseのコードを実行します");
                            }
                            if let ExecutionMode::Interactive = self.execution_mode {
                                self.execution_mode = ExecutionMode::Debug
                            }
                            let status = Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .execute_block(self.else_stmt.clone());
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
                            self.else_stmt = Vec::new();
                            self.stmt = Vec::new();
                        } else {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("条件が一致したので、実行します");
                            }
                            if let ExecutionMode::Interactive = self.execution_mode {
                                self.execution_mode = ExecutionMode::Debug
                            }
                            let status = Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .execute_block(self.stmt.clone());
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
                            self.else_stmt = Vec::new();
                            self.stmt = Vec::new();
                        }
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("if") {
                    self.nest_if += 1;
                    self.stmt.push(code.to_string());

                    self.control_mode = ControlMode::If;
                } else {
                    self.else_stmt.push(code.to_string());
                }
            }
            ControlMode::While => {
                if code.contains("end while") {
                    if self.nest_while > 0 {
                        self.nest_while -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        loop {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("whileの条件式を評価します");
                            }
                            if self.compute(self.expr.trim().to_string()) == 0.0 {
                                self.stmt = Vec::new();
                                if let ExecutionMode::Script = self.execution_mode {
                                } else {
                                    println!("条件が一致しなかったので、ループを脱出します");
                                }
                                break;
                            } else {
                                if let ExecutionMode::Script = self.execution_mode {
                                } else {
                                    println!("条件が一致したので、ループを継続します");
                                }
                                if let ExecutionMode::Interactive = self.execution_mode {
                                    self.execution_mode = ExecutionMode::Debug
                                }
                                let status = Executor::new(
                                    &mut self.memory,
                                    &mut self.name_space,
                                    self.execution_mode.clone(),
                                )
                                .execute_block(self.stmt.clone());
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
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("while") {
                    self.nest_while += 1;
                } else {
                    self.stmt.push(code.to_string());
                }
            }

            ControlMode::Function => {
                if code.contains("end func") {
                    if self.nest_func > 0 {
                        self.nest_func -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        self.set_function(self.data.clone(), self.stmt.clone());
                        self.stmt = Vec::new();
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("func") {
                    self.nest_func += 1;
                } else {
                    self.stmt.push(code.to_string());
                }
            }

            ControlMode::Normal => {
                if let ExecutionMode::Debug = self.execution_mode {
                    println!("〔 {code} 〕を実行します");
                }

                if code.contains("var") {
                    // 変数の定義
                    let new_code = code.replacen("var", "", 1);
                    let code = &new_code;
                    let params: Vec<&str> = code.split("=").collect();
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("変数{}を定義します", params[0].trim().replace(" ", ""));
                    }

                    self.set_variable(
                        params[0].trim().replace(" ", ""),
                        params[1..].join("=").to_string(),
                    );
                } else if code.contains("func") {
                    //　関数の定義
                    if !code.contains("(") {
                        println!("エラー! 関数にはカッコをつけてください");
                        return None;
                    }
                    self.data = code.to_string();
                    let name: &String = &code
                        .trim()
                        .replacen("func", "", 1)
                        .replace(")", "")
                        .replace(" ", "")
                        .replace("　", "")
                        .split("(")
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()[0];

                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("関数{}を定義します", name);
                    }
                    self.control_mode = ControlMode::Function;
                } else if code.contains("call") {
                    // 関数呼び出し
                    self.call_function(code.to_string());
                } else if code.contains("for") {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("ループ回数を求めます");
                    }
                    let new_code = code.replacen("for", "", 1);
                    self.count = self.compute(new_code).round() as usize; // ループ回数

                    self.control_mode = ControlMode::For;
                } else if code.contains("if") {
                    let new_code = code.replacen("if", "", 1);
                    self.expr = new_code;

                    self.control_mode = ControlMode::If
                } else if code.contains("while") {
                    let new_code = code.replacen("while", "", 1);
                    self.expr = new_code;

                    self.control_mode = ControlMode::While;
                } else if code.contains("input") {
                    // 標準入力
                    let new_code = code.replacen("input", "", 1);
                    let name = &new_code;
                    let inputed = if let ExecutionMode::Script = self.execution_mode {
                        input("> ")
                    } else {
                        println!("標準入力を受け取ります");
                        input("[入力]> ")
                    };
                    self.set_variable(name.trim().replace(" ", ""), inputed.to_string());
                } else if code.contains("print") {
                    //　標準出力
                    let new_code = code.replacen("print", "", 1);
                    let mut text = String::new();
                    let params = &new_code;
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("標準出力に表示します");
                    }
                    let elements: Vec<String> = {
                        let mut elements = Vec::new();
                        let mut buffer = String::new();
                        let mut stack = 0;

                        for c in params.chars() {
                            match c {
                                '(' => {
                                    stack += 1;
                                    buffer.push('(');
                                }
                                ')' => {
                                    stack -= 1;
                                    buffer.push(')');
                                }
                                ',' if stack == 0 => {
                                    elements.push(buffer.clone());
                                    buffer.clear();
                                }
                                _ => {
                                    buffer.push(c);
                                }
                            }
                        }

                        if !buffer.is_empty() {
                            elements.push(buffer);
                        }

                        elements
                    };
                    for i in elements {
                        if i.contains("'") || i.contains("\"") {
                            //文字列か？
                            text += &i.replace("'", "").replace("\"", "");
                        } else {
                            //文字列以外は式として扱われる
                            text += self.compute(i.trim().to_string()).to_string().as_str();
                        }
                    }
                    if let ExecutionMode::Script = self.execution_mode {
                        println!("{text}");
                    } else {
                        println!("[出力]: {text}");
                    }
                } else if code.contains("ref") {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("変数の参照を取得します")
                    }
                    let code = code.replacen("ref", "", 1);
                    if code.contains("=") {
                        let params: Vec<&str> = code.split("=").collect();
                        let address = self.reference_variable(params[1..].join("=").to_string());
                        if let Some(i) = address {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("変数{}のアドレスは{}です", params[1], i);
                            }
                            self.set_variable(params[0].to_string(), i.to_string());
                        }
                    } else {
                        let address = self.reference_variable(code.clone());
                        if let Some(i) = address {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("変数{}のアドレスは{}です", code, i);
                            }
                        }
                    }
                } else if code.contains("mem") {
                    let mut name_max_len = 0;
                    for i in self.memory.iter() {
                        if name_max_len < i.name.len() {
                            name_max_len = i.name.len()
                        }
                    }

                    let mut value_max_len = 0;
                    for i in self.memory.iter() {
                        if value_max_len < i.value.to_string().len() {
                            value_max_len = i.value.to_string().len()
                        }
                    }

                    if !self.memory.is_empty() {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("+-- メモリ内の変数");
                        }
                        for i in 0..self.memory.len() {
                            let vars = &self.memory[i];
                            println!(
                                "| [{:>3}] {:<name_max_len$} : {:>value_max_len$} ",
                                i, vars.name, vars.value
                            )
                        }
                    } else {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("変数がありません");
                        }
                    }
                    if !self.name_space.is_empty() {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("+-- メモリ内の関数");
                        }
                        for i in self.name_space.iter() {
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("| +--  {} ({}) ", i.name, i.args.join(", "));
                            }
                            let mut number = 0; //行数
                            for j in i.code.iter() {
                                if j != "" {
                                    number += 1;
                                    if let ExecutionMode::Script = self.execution_mode {
                                    } else {
                                        println!(
                                            "| | {number:>len$}: {j}",
                                            len = i.code.len().to_string().len()
                                        );
                                    }
                                }
                            }
                        }
                    } else {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("関数がありません");
                        }
                    }
                } else if code.contains("del") {
                    // 変数や関数の削除
                    let new_code = code.replacen("del", "", 1);
                    let name = &new_code;
                    if name.contains("(") {
                        if let Some(index) = self.reference_function(name.to_owned()) {
                            self.name_space.remove(index);
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("関数{}を削除しました", name);
                            }
                        }
                    } else {
                        match self.reference_variable(name.clone()) {
                            Some(index) => {
                                self.memory.remove(index);
                                if let ExecutionMode::Script = self.execution_mode {
                                } else {
                                    println!("変数{}を削除しました", name);
                                }
                            }
                            None => {}
                        }
                    }
                } else if code.contains("rand") {
                    // 乱数
                    let new_code = code.replacen("rand", "", 1);
                    let params = new_code.split(",").collect::<Vec<&str>>();
                    if params.len() < 3 {
                        let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                        let temp: i64 = rng.gen_range(1, 10);
                        self.set_variable(params[0].trim().replace(" ", ""), temp.to_string());
                    } else {
                        let mut rng = rand::thread_rng(); // デフォルトの乱数生成器を初期化します
                        let temp: i64 = rng.gen_range(
                            self.compute({
                                if let ExecutionMode::Script = self.execution_mode {
                                } else {
                                    println!("最小値を求めます");
                                }
                                String::from(params[1])
                            })
                            .round() as i64,
                            self.compute({
                                if let ExecutionMode::Script = self.execution_mode {
                                } else {
                                    println!("最大値を求めます");
                                }
                                String::from(params[2])
                            })
                            .round() as i64,
                        );
                        self.set_variable(params[0].trim().replace(" ", ""), temp.to_string());
                    }
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("乱数を生成しました");
                    }
                } else if code.contains("return") {
                    let return_value = code.replacen("return", "", 1);
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("戻り値を返します");
                    }
                    return Some(self.compute(return_value));
                } else if code.contains("break") {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("ループを脱出します");
                    }
                    return Some(f64::MAX); //ステータスコード
                } else if code == "exit" {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("プロセスを終了します");
                    }
                    exit(0);
                } else if code == "" {
                } else {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("コマンドが不正です: {}", code)
                    }
                }
            }
        }
        return None;
    }

    /// ブロックを実行
    pub fn execute_block(&mut self, code: Vec<String>) -> Option<f64> {
        let mut number = 0;
        for lin in code.iter() {
            let lin = lin.trim().split("#").collect::<Vec<&str>>()[0];
            if lin == "" {
                continue;
            }

            if let ControlMode::Normal = self.control_mode {
                if let ExecutionMode::Script = self.execution_mode {
                } else {
                    number = number + 1;
                    print!("{number}行目の");
                }
            }
            if let Some(i) = self.execute(lin.to_string()) {
                // 戻り値を返す
                return Some(i);
            }

            if code.iter().position(|x| x == lin) != Some(code.len() - 1) {
                if lin != "" {
                    if let ControlMode::Normal = self.control_mode {
                        if let ExecutionMode::Debug = self.execution_mode {
                            self.debug_menu();
                        }
                    }
                }
            }
        }
        return None;
    }

    /// REPLで対話的に実行する
    pub fn interactive(&mut self) {
        loop {
            self.execution_mode = ExecutionMode::Interactive;
            let code = input(
                format!(
                    "{}> ",
                    match self.control_mode {
                        ControlMode::If => "If分岐",
                        ControlMode::Else => "Else分岐",
                        ControlMode::For => "Forループ",
                        ControlMode::While => "Whileループ",
                        ControlMode::Normal => "プログラム",
                        ControlMode::Function => "関数定義",
                    }
                )
                .as_str(),
            );
            self.execute(code);
        }
    }

    /// スクリプトを実行する
    pub fn script(&mut self, code: &String) -> Option<f64> {
        self.execution_mode = ExecutionMode::Script;
        return self.execute_block(code.split("\n").map(|x| x.to_string()).collect());
    }

    /// ファイルをデバッグする
    pub fn debugger(&mut self, code: &String) -> Option<f64> {
        self.execution_mode = ExecutionMode::Debug;
        self.execute_block(code.split("\n").map(|x| x.to_string()).collect())
    }

    // デバッグメニューを表示する
    fn debug_menu(&mut self) {
        loop {
            let menu = input("デバッグメニュー>>> ");
            if menu.contains("var") {
                let lim = &menu.replacen("var", "", 1);
                let params: Vec<&str> = lim.split("=").collect();

                self.set_variable(
                    params[0].trim().replace(" ", ""),
                    params[1..].join("=").to_string(),
                );
            } else if menu.contains("del") {
                // 変数や関数の削除
                let new_lines = menu.replacen("del", "", 1);
                let name = &new_lines;
                if name.contains("(") {
                    if let Some(index) = self.reference_function(name.to_owned()) {
                        self.name_space.remove(index);
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("関数{}を削除しました", name);
                        }
                    }
                } else {
                    match self.reference_variable(name.clone()) {
                        Some(index) => {
                            self.memory.remove(index);
                            if let ExecutionMode::Script = self.execution_mode {
                            } else {
                                println!("変数{}を削除しました", name);
                            }
                        }
                        None => {}
                    }
                }
            } else if menu.contains("ref") {
                if let ExecutionMode::Script = self.execution_mode {
                } else {
                    println!("変数の参照を取得します")
                }
                let lines = menu.replacen("ref", "", 1);
                if lines.contains("=") {
                    let params: Vec<&str> = lines.split("=").collect();
                    let address = self.reference_variable(params[1..].join("=").to_string());
                    if let Some(i) = address {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("変数{}のアドレスは{}です", params[0], i);
                        }
                        self.set_variable(params[0].to_string(), i.to_string());
                    }
                } else {
                    let address = self.reference_variable(lines.clone());
                    if let Some(i) = address {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("変数{}のアドレスは{}です", lines, i);
                        }
                    }
                }
            } else if menu.contains("mem") {
                let mut name_max_len = 0;
                for i in self.memory.iter() {
                    if name_max_len < i.name.len() {
                        name_max_len = i.name.len()
                    }
                }

                let mut value_max_len = 0;
                for i in self.memory.iter() {
                    if value_max_len < i.value.to_string().len() {
                        value_max_len = i.value.to_string().len()
                    }
                }

                if !self.memory.is_empty() {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("+-- メモリ内の変数");
                    }
                    for i in 0..self.memory.len() {
                        let vars = &self.memory[i];
                        println!(
                            "| [{:>3}] {:<name_max_len$} : {:>value_max_len$} ",
                            i, vars.name, vars.value
                        )
                    }
                } else {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("変数がありません");
                    }
                }

                if !self.name_space.is_empty() {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("+-- メモリ内の関数");
                    }
                    for i in self.name_space.iter() {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("| +--  {} ({}) ", i.name, i.args.join(", "));
                        }
                        let mut number = 0; //行数
                        for j in i.code.iter() {
                            if j != "" {
                                number += 1;
                                if let ExecutionMode::Script = self.execution_mode {
                                } else {
                                    println!(
                                        "| | {number:>len$}: {j}",
                                        len = i.code.len().to_string().len()
                                    );
                                }
                            }
                        }
                    }
                } else {
                    if let ExecutionMode::Script = self.execution_mode {
                    } else {
                        println!("関数がありません");
                    }
                }
            } else if menu.contains("exit") {
                input("デバッグを中断します");
                exit(0);
            } else {
                println!("継続します");
                break;
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
                if let ExecutionMode::Script = self.execution_mode {
                } else {
                    println!("変数{name}が見つかりません");
                }
                None
            }
        }
    }

    ///　関数を呼び出す
    fn call_function(&mut self, item: String) -> f64 {
        if !item.contains("(") {
            println!("エラー! 関数にはカッコをつけてください");
            return 0.0;
        }
        let new_lines: Vec<String> = item
            .trim()
            .replacen("call", "", 1)
            .replace(")", "")
            .split("(")
            .map(|s| s.to_string())
            .collect();
        let func_name: String = new_lines[0].replace(" ", "").replace("　", "").clone();

        if let ExecutionMode::Script = self.execution_mode {
        } else {
            println!("引数の値を求めます");
        }

        let args_value: Vec<f64> = new_lines[1]
            .split(',')
            .map(|s| self.compute(s.to_string()))
            .collect();

        let name = func_name
            .replace(" ", "")
            .replace("　", "")
            .replace("(", "")
            .replace(")", "");

        if let ExecutionMode::Script = self.execution_mode {
        } else {
            println!("関数{name}を呼び出します");
        }

        let code = match self.get_function(name.clone()) {
            Some(func) => func.code,
            None => return 0.0,
        };

        let function_args = self.get_function(name.clone()).unwrap().args.clone();

        let mut pre = Vec::new();

        for i in function_args.iter() {
            pre.push(format!("var {i}")); // 引数は変数として扱われる
        }

        for (i, j) in function_args.iter().zip(args_value.iter()) {
            pre.push(format!("var {i} = {j}")); // 引数は変数として扱われる
        }

        let mut instance = Executor::new(
            &mut self.memory,
            &mut self.name_space,
            ExecutionMode::Script,
        );

        instance.execute_block(pre);
        instance.execution_mode = self.execution_mode.clone();

        match instance.execute_block(code) {
            Some(indes) => indes,
            None => 0.0,
        }
    }

    /// 関数を定義する
    fn set_function(&mut self, item: String, code: Vec<String>) {
        let new_lines: Vec<String> = item
            .trim()
            .replacen("func", "", 1)
            .replace(")", "")
            .replace(" ", "")
            .replace("　", "")
            .split("(")
            .map(|s| s.to_string())
            .collect();

        let func_name: String = new_lines[0].clone();

        let args_value: Vec<String> = new_lines[1].split(',').map(|s| s.to_string()).collect();

        let name = func_name
            .replace(" ", "")
            .replace("　", "")
            .replace("(", "")
            .replace(")", "");

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
            let address = self.reference_function(func_name.clone()).unwrap_or(0);
            self.name_space[address].code = code.clone();
            self.name_space[address].args = args_value.clone();
            if let ExecutionMode::Script = self.execution_mode {
            } else {
                println!("関数{name}のデータを更新しました");
            }
        } else {
            //ない場合は新規に確保する
            self.name_space.push(Function {
                name: name.clone(),
                args: args_value,
                code: code,
            });
            if let ExecutionMode::Script = self.execution_mode {
            } else {
                println!("メモリに関数を保存しました");
            }
        }
    }

    /// 関数の参照
    fn reference_function(&mut self, name: String) -> Option<usize> {
        let name = name
            .trim()
            .replace(" ", "")
            .replace("　", "")
            .replace("(", "")
            .replace(")", "");
        match self
            .name_space
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
        {
            Some(index) => Some(index),
            None => {
                if let ExecutionMode::Script = self.execution_mode {
                } else {
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
            let address = self.reference_variable(name.clone()).unwrap_or(0);
            if let ExecutionMode::Script = self.execution_mode {
            } else {
                println!("値を求めます");
            }
            self.memory[address].value = self.compute(expr.clone());
            if let ExecutionMode::Script = self.execution_mode {
            } else {
                println!("変数{name}のデータを更新しました");
            }
        } else {
            //ない場合は新規に確保する
            if let ExecutionMode::Script = self.execution_mode {
            } else {
                println!("値を求めます");
            }
            let value = self.compute(expr.clone());
            self.memory.push(Variable {
                name: name.clone(),
                value,
            });
            if let ExecutionMode::Script = self.execution_mode {
            } else {
                println!("メモリに変数を確保しました");
            }
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
        if let ExecutionMode::Script = self.execution_mode {
        } else {
            println!("変数{name}を読み込みます");
        }
        match self.get_variable(name) {
            Some(i) => i.value,
            None => return 0.0,
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
        let tokens: Vec<String> = {
            let mut elements = Vec::new();
            let mut buffer = String::new();
            let mut stack = 0;

            for c in expr.chars() {
                match c {
                    '(' => {
                        stack += 1;
                        buffer.push('(');
                    }
                    ')' => {
                        stack -= 1;
                        buffer.push(')');
                    }
                    ' ' | '　' if stack == 0 => {
                        elements.push(buffer.clone());
                        buffer.clear();
                    }
                    _ => {
                        buffer.push(c);
                    }
                }
            }

            if !buffer.is_empty() {
                elements.push(buffer);
            }

            elements
        };
        let mut stack: Vec<f64> = Vec::new();
        if let ExecutionMode::Script = self.execution_mode {
        } else {
            println!("+-- 式を計算します");
        }
        for item in tokens {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            if let ExecutionMode::Script = self.execution_mode {
            } else {
                println!("| Stack: {:?}  ←  '{}'", stack, item);
            }

            if item.contains("(") {
                stack.push(self.call_function(item.to_string()));
                continue;
            }
            match item.parse::<f64>() {
                Ok(number) => {
                    stack.push(number);
                    continue;
                }
                Err(_) => {
                    let y = stack.pop();
                    let x = stack.pop();
                    match item {
                        "+" => stack.push(x.unwrap_or(0.0) + y.unwrap_or(0.0)),
                        "-" => stack.push(x.unwrap_or(0.0) - y.unwrap_or(0.0)),
                        "*" => stack.push(x.unwrap_or(0.0) * y.unwrap_or(0.0)),
                        "/" => stack.push(x.unwrap_or(0.0) / y.unwrap_or(0.0)),
                        "%" => stack.push(x.unwrap_or(0.0) % y.unwrap_or(0.0)),
                        "^" => stack.push(x.unwrap_or(0.0).powf(y.unwrap_or(0.0))),
                        "=" => stack.push(if x == y { 1.0 } else { 0.0 }),
                        "&" => stack.push(if x.unwrap_or(0.0) != 0.0 && y.unwrap_or(0.0) != 0.0 {
                            1.0 // 論理値はfalseを0.0,trueを1.0として表す
                        } else {
                            0.0
                        }),
                        "|" => stack.push(if x.unwrap_or(0.0) != 0.0 || y.unwrap_or(0.0) != 0.0 {
                            1.0
                        } else {
                            0.0
                        }),
                        ">" => stack.push(if x > y { 1.0 } else { 0.0 }),
                        "<" => stack.push(if x < y { 1.0 } else { 0.0 }),
                        "!" => {
                            if let Some(i) = x {
                                stack.push(i);
                            }
                            stack.push(if y.unwrap_or(0.0) == 0.0 { 1.0 } else { 0.0 })
                        }
                        "~" => {
                            if let Some(i) = x {
                                stack.push(i);
                            }
                            stack.push({
                                if let ExecutionMode::Script = self.execution_mode {
                                } else {
                                    println!("ポインタがさす値を求めます");
                                }
                                if y.unwrap_or(0.0).round() as usize > &self.memory.len() - 1 {
                                    println!("エラー!アドレスが不正です");
                                    0.0
                                } else {
                                    self.memory[y.unwrap_or(0.0).round() as usize].value
                                }
                            })
                        }
                        _ => {
                            if let Some(i) = x {
                                stack.push(i);
                            }
                            if let Some(i) = y {
                                stack.push(i);
                            }

                            stack.push(self.get_variable_value(item.to_string()));
                        }
                    }
                }
            };
        }
        let result = stack.pop().unwrap_or(0.0);
        if let ExecutionMode::Script = self.execution_mode {
        } else {
            println!("結果 = {}", result);
        }
        return result;
    }
}
