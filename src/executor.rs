use crate::get_file_contents;
use std::io::{self, Write};
use std::process::exit;

/// 標準入力を取得する
pub fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin().read_line(&mut result).ok();
    return result.trim().parse().ok().unwrap_or("".to_string());
}

#[derive(Clone, Debug)]
pub enum Type {
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<Type>),
    Function(usize),
}

/// 変数のデータ
#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub value: Type,
}

/// 関数のデータ
#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<String>,
    pub code: Vec<String>,
}

/// 制御モード
#[derive(Clone)]
pub enum ControlMode {
    If,
    Else,
    For,
    While,
    Function,
    Try,
    Catch,
    Normal,
}

#[derive(Clone)]
pub enum ReturnValue {
    Some(Type),
    None,
    Error(String),
    Break,
    Next,
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
    pub memory: &'a mut Vec<Variable>,     //　メモリ内の変数
    pub name_space: &'a mut Vec<Function>, // 関数の名前空間
    pub stmt: Vec<String>,                 // ブロックのステートメント
    pub else_stmt: Vec<String>,            // elseステートメント
    data: String,                          // 関数のデータ
    expr: String,                          // 条件式
    pub execution_mode: ExecutionMode,     // 制御ブロックの状態
    pub control_mode: ControlMode,         // 制御ブロックの状態
    pub nest_if: usize,                    // ifネストの階層を表す
    pub nest_try: usize,                   // tryネストの階層を表す
    pub nest_for: usize,                   // forネストの階層を表す
    pub nest_while: usize,                 // whileネストの階層を表す
    pub nest_func: usize,                  // funcネストの階層を表す
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
            data: "".to_string(),
            expr: "".to_string(),
            control_mode: ControlMode::Normal,
            execution_mode: mode,
            nest_if: 0,
            nest_try: 0,
            nest_for: 0,
            nest_while: 0,
            nest_func: 0,
        }
    }

    /// 文の実行
    pub fn execute(&mut self, code: String) -> ReturnValue {
        match self.control_mode {
            ControlMode::For => {
                if code.contains("end for") || code.contains("endfor") {
                    // ネストの階層を判別する
                    if self.nest_for > 0 {
                        self.nest_for -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        self.log_print(format!("ループ回数を求めます"));
                        let count = if let Type::Number(i) = match self.compute(self.expr.clone()) {
                            ReturnValue::Some(i) => i,
                            ReturnValue::Error(e) => return ReturnValue::Error(e),
                            _ => Type::Number(0.0),
                        } {
                            i.round() as usize
                        } else {
                            return ReturnValue::Error(
                                "エラー! ループ回数は数値型です".to_string(),
                            );
                        };
                        for i in 0..count {
                            self.log_print(format!("{}回目のループ", i + 1));

                            let status = Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .execute_block(self.stmt.clone());
                            match status {
                                ReturnValue::Break => break,
                                ReturnValue::Next => continue,
                                ReturnValue::Some(n) => return ReturnValue::Some(n),
                                ReturnValue::Error(n) => return ReturnValue::Error(n),
                                ReturnValue::None => {}
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
                } else if code.contains("end if") || code.contains("endif") {
                    if self.nest_if > 0 {
                        self.nest_if -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        self.log_print(format!("ifの条件式を評価します"));
                        if let Type::Bool(true) = match self.compute(self.expr.clone()) {
                            ReturnValue::Some(i) => i,
                            ReturnValue::Error(e) => return ReturnValue::Error(e),
                            _ => Type::Number(0.0),
                        } {
                            self.log_print(format!("条件が一致したので、実行します"));

                            let status = Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .execute_block(self.stmt.clone());
                            match status {
                                ReturnValue::Break => return ReturnValue::Break,
                                ReturnValue::Next => return ReturnValue::Next,
                                ReturnValue::Some(n) => return ReturnValue::Some(n),
                                ReturnValue::Error(n) => return ReturnValue::Error(n),
                                ReturnValue::None => {}
                            }
                            self.stmt = Vec::new();
                        } else {
                            self.log_print(format!("条件が一致しなかったので、実行しません"));
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
                if code.contains("end if") || code.contains("endif") {
                    if self.nest_if > 0 {
                        self.nest_if -= 1;
                        self.else_stmt.push(code.to_string());
                    } else {
                        self.log_print(format!("ifの条件式を評価します"));
                        if let Type::Bool(true) = match self.compute(self.expr.clone()) {
                            ReturnValue::Some(i) => i,
                            ReturnValue::Error(e) => return ReturnValue::Error(e),
                            _ => Type::Number(0.0),
                        } {
                            self.log_print(format!("条件が一致したので、実行します"));

                            let status = Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .execute_block(self.stmt.clone());
                            match status {
                                ReturnValue::Break => return ReturnValue::Break,
                                ReturnValue::Next => return ReturnValue::Next,
                                ReturnValue::Some(n) => return ReturnValue::Some(n),
                                ReturnValue::Error(n) => return ReturnValue::Error(n),
                                ReturnValue::None => {}
                            }
                            self.else_stmt = Vec::new();
                            self.stmt = Vec::new();
                        } else {
                            self.log_print(format!(
                                "条件が一致しなかったので、elseのコードを実行します"
                            ));

                            let status = Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .execute_block(self.else_stmt.clone());
                            match status {
                                ReturnValue::Break => return ReturnValue::Break,
                                ReturnValue::Next => return ReturnValue::Next,
                                ReturnValue::Some(n) => return ReturnValue::Some(n),
                                ReturnValue::Error(n) => return ReturnValue::Error(n),
                                ReturnValue::None => {}
                            }
                            self.else_stmt = Vec::new();
                            self.stmt = Vec::new();
                        }
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("if") {
                    self.nest_if += 1;
                    self.else_stmt.push(code.to_string());
                } else {
                    self.else_stmt.push(code.to_string());
                }
            }
            ControlMode::While => {
                if code.contains("end while") || code.contains("endwhile") {
                    if self.nest_while > 0 {
                        self.nest_while -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        loop {
                            self.log_print(format!("whileの条件式を評価します"));
                            if let Type::Bool(true) = match self.compute(self.expr.clone()) {
                                ReturnValue::Some(i) => i,
                                ReturnValue::Error(e) => return ReturnValue::Error(e),
                                _ => Type::Number(0.0),
                            } {
                                self.log_print(format!("条件が一致したので、ループを継続します"));
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
                                    ReturnValue::Break => break,
                                    ReturnValue::Next => continue,
                                    ReturnValue::Some(n) => return ReturnValue::Some(n),
                                    ReturnValue::Error(n) => return ReturnValue::Error(n),
                                    ReturnValue::None => {}
                                }
                            } else {
                                self.stmt = Vec::new();
                                self.log_print(format!(
                                    "条件が一致しなかったので、ループを脱出します"
                                ));
                                break;
                            }
                        }
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("while") {
                    self.nest_while += 1;
                    self.stmt.push(code.to_string());
                } else {
                    self.stmt.push(code.to_string());
                }
            }

            ControlMode::Function => {
                if code.contains("end func") || code.contains("endfunc") {
                    if self.nest_func > 0 {
                        self.nest_func -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        let token = self.tokenize_arguments(
                            self.data
                                .clone()
                                .replace(")", "")
                                .split("(")
                                .collect::<Vec<&str>>()[1],
                        );
                        self.declaration_function(Function {
                            name: self
                                .data
                                .clone()
                                .replacen("func", "", 1)
                                .split("(")
                                .collect::<Vec<&str>>()[0]
                                .trim()
                                .to_string(),
                            args: token,
                            code: self.stmt.clone(),
                        });
                        self.stmt = Vec::new();
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("func") {
                    self.nest_func += 1;
                    self.stmt.push(code.to_string());
                } else {
                    self.stmt.push(code.to_string());
                }
            }

            ControlMode::Try => {
                if code.contains("catch") {
                    if self.nest_try > 0 {
                        self.nest_try -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        self.control_mode = ControlMode::Catch;
                    }
                } else if code.contains("try") {
                    self.nest_try += 1;
                    self.stmt.push(code.to_string());
                } else {
                    self.stmt.push(code.to_string());
                }
            }

            ControlMode::Catch => {
                if code.contains("end try") || code.contains("endtry") {
                    if self.nest_try > 0 {
                        self.nest_try -= 1;
                        self.stmt.push(code.to_string());
                    } else {
                        self.log_print("エラーが起きそうなTryのコードを実行します".to_string());
                        let status = Executor::new(
                            &mut self.memory,
                            &mut self.name_space,
                            self.execution_mode.clone(),
                        )
                        .execute_block(self.stmt.clone());
                        match status {
                            ReturnValue::Break => return ReturnValue::Break,
                            ReturnValue::Next => return ReturnValue::Next,
                            ReturnValue::Some(n) => return ReturnValue::Some(n),
                            ReturnValue::Error(_) => {
                                self.log_print(
                                    "エラーが発生したので、Catchのコードを実行します".to_string(),
                                );
                                let status = Executor::new(
                                    &mut self.memory,
                                    &mut self.name_space,
                                    self.execution_mode.clone(),
                                )
                                .execute_block(self.else_stmt.clone());
                                match status {
                                    ReturnValue::Break => return ReturnValue::Break,
                                    ReturnValue::Next => return ReturnValue::Next,
                                    ReturnValue::Some(n) => return ReturnValue::Some(n),
                                    ReturnValue::Error(e) => return ReturnValue::Error(e),
                                    ReturnValue::None => {}
                                }
                            }
                            ReturnValue::None => {
                                self.log_print("正常に実行されました".to_string());
                            }
                        }
                        self.stmt = Vec::new();
                        self.else_stmt = Vec::new();
                        self.control_mode = ControlMode::Normal;
                    }
                } else if code.contains("try") {
                    self.nest_try += 1;
                    self.else_stmt.push(code.to_string());
                } else {
                    self.else_stmt.push(code.to_string());
                }
            }

            ControlMode::Normal => {
                if code.contains("var") {
                    // 変数の定義
                    let new_code = code.replacen("var", "", 1);
                    let code = &new_code;
                    let params: Vec<&str> = code.split("=").collect();
                    let name = params[0].trim();
                    let expr = params[1..].join("=").to_string();
                    if name.contains("[") {
                        let value = match self.compute(expr) {
                            ReturnValue::Some(i) => i,
                            ReturnValue::Error(e) => return ReturnValue::Error(e),
                            _ => Type::Number(0.0),
                        };
                        self.set_sequence_value(name.to_string(), value);
                    } else {
                        let name = name.replace(" ", "");
                        self.log_print(format!("変数{}を定義します", name));
                        let value = match self.compute(params[1..].join("=").to_string()) {
                            ReturnValue::Some(i) => i,
                            _ => Type::Number(0.0),
                        };
                        if let ReturnValue::Error(e) = self.set_variable(name, value) {
                            return ReturnValue::Error(e);
                        };
                    }
                } else if code.contains("func") {
                    //　関数の定義
                    if !code.contains("(") {
                        self.log_print("エラー! 関数にはカッコをつけてください".to_string());
                        return ReturnValue::Error(
                            "エラー! 関数にはカッコをつけてください".to_string(),
                        );
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

                    self.log_print(format!("関数{}を定義します", name));
                    self.control_mode = ControlMode::Function;
                } else if code.contains("for") {
                    let new_code = code.replacen("for", "", 1);
                    self.expr = new_code;
                    self.control_mode = ControlMode::For;
                } else if code.contains("if") {
                    self.expr = code.replacen("if", "", 1);
                    self.control_mode = ControlMode::If
                } else if code.contains("try") {
                    self.control_mode = ControlMode::Try
                } else if code.contains("while") {
                    self.expr = code.replacen("while", "", 1);
                    self.control_mode = ControlMode::While;
                } else if code.contains("import") {
                    let code = code.replacen("import", "", 1);
                    self.log_print(format!("モジュール{code}を読み込みます"));
                    let module = match get_file_contents(code) {
                        Ok(code) => code,
                        Err(e) => return ReturnValue::Error(format!("エラー! {e}")),
                    };

                    let mode = self.execution_mode.clone();
                    match self.execution_mode {
                        ExecutionMode::Script => {
                            self.script(&module);
                        }
                        ExecutionMode::Debug | ExecutionMode::Interactive => {
                            self.debugger(&module);
                        }
                    }
                    self.execution_mode = mode;
                } else if code.contains("mem") {
                    self.show_memory()
                } else if code.contains("del") {
                    // 変数や関数の削除
                    let new_code = code.replacen("del", "", 1);
                    let name = &new_code;
                    if name.contains("(") {
                        if let Some(index) = self.reference_function(name.to_owned()) {
                            self.name_space.remove(index);
                            self.log_print(format!("関数{}を削除しました", name));
                        } else {
                            self.log_print(format!("関数{name}が見つかりません"));
                        }
                    } else {
                        if name.contains("[") {
                            self.del_sequence_value(name.to_string());
                        } else {
                            match self.reference_variable(name.clone()) {
                                Some(index) => {
                                    self.memory.remove(index);
                                    self.log_print(format!("変数{}を削除しました", name));
                                }
                                None => {
                                    self.log_print(format!("変数{name}が見つかりません"));
                                }
                            }
                        }
                    }
                } else if code.contains("return") {
                    let return_value = code.replacen("return", "", 1);
                    self.log_print(format!("戻り値を返します"));
                    return self.compute(return_value);
                } else if code.contains("break") {
                    self.log_print(format!("ループを脱出します"));
                    return ReturnValue::Break;
                } else if code.contains("next") {
                    self.log_print(format!("次のループへ移ります"));
                    return ReturnValue::Next; //ステータスコード
                } else if code == "exit" {
                    self.log_print(format!("プロセスを終了します"));
                    exit(0)
                } else {
                    match self.compute(code) {
                        ReturnValue::Error(e) => return ReturnValue::Error(e),
                        _ => {}
                    };
                }
            }
        }
        return ReturnValue::None;
    }

    /// ブロックを実行
    pub fn execute_block(&mut self, code: Vec<String>) -> ReturnValue {
        let mut number = 0;
        for lin in code.iter() {
            let lin = if let ControlMode::Normal = self.control_mode {
                let params = lin.trim().split("#").collect::<Vec<&str>>();
                if params.len() > 1 {
                    if !params[1..]
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join("#")
                        .is_empty()
                    {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!(
                                "※ コメント「{}」",
                                params[1..]
                                    .iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join("#")
                            );
                        }
                    }
                }
                params[0]
            } else {
                lin.as_str()
            };
            if lin.trim().split("#").collect::<Vec<&str>>()[0].is_empty() {
                continue;
            }

            if let ControlMode::Normal = self.control_mode {
                if let ExecutionMode::Script = self.execution_mode {
                } else {
                    number = number + 1;
                    println!("{number}行目の〔 {lin} 〕を実行します");
                }
            }

            let status = self.execute(lin.to_string());

            if let ControlMode::Normal = self.control_mode {
                if let ExecutionMode::Script = self.execution_mode {
                } else {
                    self.debug_menu();
                }
            }

            match status {
                ReturnValue::Some(i) => return ReturnValue::Some(i),
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                ReturnValue::Next => return ReturnValue::Next,
                ReturnValue::Break => return ReturnValue::Break,
                ReturnValue::None => {}
            }
        }
        return ReturnValue::None;
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
                        ControlMode::Try => "Try文",
                        ControlMode::Catch => "Catch文",
                        ControlMode::For => "Forループ",
                        ControlMode::While => "Whileループ",
                        ControlMode::Normal => "プログラム",
                        ControlMode::Function => "関数定義",
                    }
                )
                .as_str(),
            );
            let code = if let ControlMode::Normal = self.control_mode {
                let params = code.trim().split("#").collect::<Vec<&str>>();
                if params.len() > 1 {
                    if !params[1..]
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join("#")
                        .is_empty()
                    {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!(
                                "※ コメント「{}」",
                                params[1..]
                                    .iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join("#")
                            );
                        }
                    }
                }
                params[0]
            } else {
                code.as_str()
            };
            if code.trim().is_empty() {
                continue;
            }

            self.execute(code.to_string());
        }
    }

    /// スクリプトを実行する
    pub fn script(&mut self, code: &String) -> ReturnValue {
        self.execution_mode = ExecutionMode::Script;
        let code: Vec<String> = code.split("\n").map(|x| x.to_string()).collect();
        self.execute_block(code)
    }

    /// ファイルをデバッグする
    pub fn debugger(&mut self, code: &String) -> ReturnValue {
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

                let value = match self.compute(params[1..].join("=").to_string()) {
                    ReturnValue::Some(i) => i,
                    _ => Type::Number(0.0),
                };

                self.set_variable(params[0].trim().replace(" ", ""), value);
            } else if menu.contains("del") {
                // 変数や関数の削除
                let new_lines = menu.replacen("del", "", 1);
                let name = &new_lines;
                if name.contains("(") {
                    if let Some(index) = self.reference_function(name.to_owned()) {
                        self.name_space.remove(index);
                        self.log_print(format!("関数{}を削除しました", name));
                    }
                } else {
                    match self.reference_variable(name.clone()) {
                        Some(index) => {
                            self.memory.remove(index);
                            self.log_print(format!("変数{}を削除しました", name));
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
                        self.log_print(format!("変数{}のアドレスは{}です", params[0], i));
                        let value = match self.compute(i.to_string()) {
                            ReturnValue::Some(v) => v,
                            _ => Type::Number(0.0),
                        };
                        self.set_variable(params[0].to_string(), value);
                    }
                } else {
                    let address = self.reference_variable(lines.clone());
                    if let Some(i) = address {
                        self.log_print(format!("変数{}のアドレスは{}です", lines, i));
                    }
                }
            } else if menu.contains("mem") {
                self.show_memory();
            } else if menu.contains("exit") {
                input("デバッグを中断します");
                exit(0)
            } else {
                println!("継続します");
                break;
            }
        }
    }

    /// 実行過程をログ表示
    pub fn log_print(&self, text: String) {
        if let ExecutionMode::Script = self.execution_mode {
        } else {
            println!("{text}");
        }
    }

    /// 変数の参照
    pub fn reference_variable(&mut self, name: String) -> Option<usize> {
        let name = name.trim().replace(" ", "").replace("　", "");
        self.memory.iter().position(|x| x.name == name)
    }

    /// シーケンス型の値を得る
    fn get_sequence_value(&mut self, item: Type, index: String) -> ReturnValue {
        if let Type::List(ref sequence) = item {
            let index = if let Type::Number(i) = match self.compute(index.clone()) {
                ReturnValue::Some(i) => i,
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                _ => Type::Number(0.0),
            } {
                let j = i as usize;
                self.log_print(format!("インデックス{i}の値を求めます"));
                if j < sequence.len() {
                    j
                } else {
                    self.log_print(format!("エラー! {i}はインデックス範囲外です"));
                    return ReturnValue::Error(format!("エラー! {i}はインデックス範囲外です"));
                }
            } else {
                if let Type::String(is) = match self.compute(index.clone()) {
                    ReturnValue::Some(i) => i,
                    ReturnValue::Error(e) => return ReturnValue::Error(e),
                    _ => Type::Number(0.0),
                } {
                    self.log_print(format!("リストの長さを求めます"));
                    if is.contains("len") {
                        if let Type::List(l) = item {
                            return ReturnValue::Some(Type::Number(l.clone().len() as f64));
                        }
                    }
                }

                self.log_print(format!("エラー! インデックスは数値型です"));
                return ReturnValue::Error(format!("エラー! インデックスは数値型です"));
            };
            return ReturnValue::Some(sequence[index].clone());
        }
        if let Type::String(ref string) = item {
            let index: usize = if let Type::Number(i) = match self.compute(index.clone()) {
                ReturnValue::Some(i) => i,
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                _ => Type::Number(0.0),
            } {
                let j: usize = i as usize;
                self.log_print(format!("'{string}'のインデックス{i}の文字を求めます"));
                if j < string.chars().count() {
                    j
                } else {
                    self.log_print(format!("エラー! {i}はインデックス範囲外です"));
                    return ReturnValue::Error(format!("エラー! {i}はインデックス範囲外です"));
                }
            } else {
                if let Type::String(is) = match self.compute(index.clone()) {
                    ReturnValue::Some(i) => i,
                    ReturnValue::Error(e) => return ReturnValue::Error(e),
                    _ => Type::Number(0.0),
                } {
                    self.log_print(format!("文字列の長さを求めます"));
                    if is.contains("len") {
                        if let Type::String(m) = item {
                            return ReturnValue::Some(Type::Number(
                                m.clone().chars().count() as f64
                            ));
                        }
                    }
                }

                self.log_print(format!("エラー! インデックスは数値型です"));
                return ReturnValue::Error(format!("エラー! インデックスは数値型です"));
            };
            return ReturnValue::Some(Type::String(
                string.chars().collect::<Vec<char>>()[index].to_string(),
            ));
        } else {
            return ReturnValue::Some(match self.compute(index.clone()) {
                ReturnValue::Some(i) => i,
                ReturnValue::Error(e) => return ReturnValue::Error(e),
                _ => Type::Number(0.0),
            });
        }
    }

    /// シーケンス型の値をセットする
    fn set_sequence_value(&mut self, item: String, value: Type) -> ReturnValue {
        let new_lines: Vec<String> = item
            .trim()
            .replace("]", "")
            .split("[")
            .map(|s| s.to_string())
            .collect();
        let name: String = new_lines[0].trim().to_string();
        let address = self.reference_variable(name.clone()).unwrap_or(0);
        let index = self.compute(new_lines[1].clone());

        match self.get_variable_value(name.clone()) {
            Type::List(mut sequence) => {
                let len = sequence.len();
                sequence[if let Type::Number(i) = match index {
                    ReturnValue::Some(i) => i,
                    ReturnValue::Error(e) => return ReturnValue::Error(e),
                    _ => Type::Number(0.0),
                } {
                    let i = i as usize;
                    self.log_print(format!("インデックス{i}の値を変更します"));
                    if i < len {
                        i
                    } else {
                        self.log_print(format!("エラー! {i}はインデックス範囲外です"));
                        return ReturnValue::Error(format!("エラー! {i}はインデックス範囲外です"));
                    }
                } else {
                    self.log_print(format!("エラー! インデックスは数値型です"));
                    return ReturnValue::Error(format!("エラー! インデックスは数値型です"));
                }] = value.clone();
                self.memory[address].value = Type::List(sequence);
            }
            Type::String(string) => {
                let mut vec = string
                    .chars()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                vec[if let Type::Number(i) = match index {
                    ReturnValue::Some(i) => i,
                    ReturnValue::Error(e) => return ReturnValue::Error(e),
                    _ => Type::Number(0.0),
                } {
                    let i: usize = i as usize;
                    self.log_print(format!("'{string}'のインデックス{i}の値を変更します"));
                    if i < string.chars().count() {
                        i
                    } else {
                        self.log_print(format!("エラー! {i}はインデックス範囲外です"));
                        return ReturnValue::Error(format!("エラー! {i}はインデックス範囲外です"));
                    }
                } else {
                    0
                }] = self.type_string(value);
                self.memory[address].value = Type::String(vec.join(""));
            }
            _ => {
                return ReturnValue::Error(format!(
                    "エラー! スライスは文字列型やリスト型のみ有効です"
                ))
            }
        }
        return ReturnValue::None;
    }

    /// シーケンス型の値を削除する
    fn del_sequence_value(&mut self, item: String) -> ReturnValue {
        let new_lines: Vec<String> = item
            .trim()
            .replace("]", "")
            .split("[")
            .map(|s| s.to_string())
            .collect();
        let name: String = new_lines[0].trim().to_string();
        if let Type::List(mut sequence) = self.get_variable_value(name.clone()) {
            let len = sequence.len();
            sequence.remove(
                if let Type::Number(i) = match self.compute(new_lines[1].clone()) {
                    ReturnValue::Some(i) => i,
                    ReturnValue::Error(e) => return ReturnValue::Error(e),
                    _ => Type::Number(0.0),
                } {
                    let j = i as usize;
                    self.log_print(format!("インデックス{i}の値を削除します"));
                    if j < len {
                        j
                    } else {
                        self.log_print(format!("エラー! {i}はインデックス範囲外です"));
                        return ReturnValue::Error(format!("エラー! {i}はインデックス範囲外です"));
                    }
                } else {
                    self.log_print(format!("エラー! インデックスは数値型です"));
                    return ReturnValue::Error(format!("エラー! インデックスは数値型です"));
                },
            );
            let address = self.reference_variable(name.clone()).unwrap_or(0);
            self.memory[address].value = Type::List(sequence);
        }
        if let Type::String(string) = self.get_variable_value(name.clone()) {
            let len = string.len();
            let mut vec = string
                .chars()
                .map(|x| x.to_string())
                .collect::<Vec<String>>();
            vec.remove(
                if let Type::Number(i) = match self.compute(new_lines[1].clone()) {
                    ReturnValue::Some(i) => i,
                    ReturnValue::Error(e) => return ReturnValue::Error(e),
                    _ => Type::Number(0.0),
                } {
                    let j = i as usize;
                    self.log_print(format!("インデックス{i}の値を削除します"));
                    if j < len {
                        j
                    } else {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            return ReturnValue::Error(format!(
                                "エラー! {i}はインデックス範囲外です"
                            ));
                        };
                        0
                    }
                } else {
                    self.log_print(format!("エラー! インデックスは数値型です"));
                    return ReturnValue::Error(format!("エラー! インデックスは数値型です"));
                },
            );
            let address = self.reference_variable(name.clone()).unwrap_or(0);
            self.memory[address].value = Type::String(vec.join(""));
        }
        return ReturnValue::None;
    }

    /// 引数のトークンを整える
    pub fn tokenize_arguments(&mut self, expr: &str) -> Vec<String> {
        let mut elements = Vec::new();
        let mut buffer = String::new();
        let mut in_quotes = false;
        let mut in_brackets = 0;
        let mut in_parentheses = 0;

        for c in expr.chars() {
            match c {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    buffer.push('"');
                }
                '"' | '\'' if in_quotes => {
                    in_quotes = false;
                    buffer.push('"');
                }
                '(' if !in_quotes => {
                    in_brackets += 1;
                    buffer.push('(');
                }
                ')' if !in_quotes => {
                    in_brackets -= 1;
                    buffer.push(')');
                }
                '[' if !in_quotes => {
                    in_parentheses += 1;
                    buffer.push('[');
                }
                ']' if !in_quotes => {
                    in_parentheses -= 1;
                    buffer.push(']');
                }
                ',' if !in_quotes && in_brackets == 0 && in_parentheses == 0 => {
                    if !buffer.is_empty() {
                        elements.push(buffer.clone());
                        buffer.clear();
                    }
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
    }

    /// 関数の参照
    pub fn reference_function(&mut self, name: String) -> Option<usize> {
        let name = name
            .trim()
            .replace(" ", "")
            .replace("　", "")
            .replace("(", "")
            .replace(")", "");
        let mut name_space = self.name_space.clone();
        name_space.reverse();
        name_space
            .iter()
            .position(|x| x.name == name.trim().replace(" ", ""))
    }

    /// 変数の値をセットする
    pub fn set_variable(&mut self, name: String, value: Type) -> ReturnValue {
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
            self.log_print(format!("値を求めます"));
            self.memory[address].value = value;
            self.log_print(format!("変数データを更新しました"));
        } else {
            //ない場合は新規に確保する
            self.log_print(format!("値を求めます"));
            self.memory.push(Variable {
                name: name.clone(),
                value: value,
            });
            self.log_print(format!("メモリに変数を確保しました"));
        }
        return ReturnValue::None;
    }

    /// 変数を取得する
    fn get_variable(&mut self, name: String) -> Option<Variable> {
        let name = name.trim().replace(" ", "").replace("　", "");
        let index = match self.reference_variable(name.clone()) {
            Some(i) => i,
            None => {
                return {
                    self.log_print(format!("変数{name}が見つかりません"));
                    None
                }
            }
        };
        return Some(self.memory[index].clone());
    }

    /// 変数の値を取得する
    fn get_variable_value(&mut self, name: String) -> Type {
        self.log_print(format!("変数{name}を読み込みます"));
        match self.get_variable(name) {
            Some(i) => i.value,
            None => Type::Number(0.0),
        }
    }

    /// データの文字列表記
    pub fn type_string(&mut self, data: Type) -> String {
        match data {
            Type::String(s) => format!("'{s}'"),
            Type::Number(i) => format!("{}", i.to_string()),
            Type::List(l) => format!(
                "[{}]",
                l.into_iter()
                    .map(|x| self.type_string(x))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Type::Bool(b) => b.to_string(),
            Type::Function(f) => {
                let func = self.function_propaty(f);
                let mut text = String::new();
                text += &format!("| +--  {} ({}) ", func.name, func.args.join(", "));
                let mut number = 0; //行数
                for j in func.code.iter() {
                    if j != "" {
                        number += 1;
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            text += &format!(
                                "| | {number:>len$}: {j}",
                                len = func.code.len().to_string().len()
                            );
                        }
                    }
                }
                text
            }
        }
    }

    /// 式の計算
    pub fn compute(&mut self, expr: String) -> ReturnValue {
        /// 式をトークンに分ける
        fn tokenize_expression(expr: &str) -> Vec<String> {
            let mut elements = Vec::new();
            let mut buffer = String::new();
            let mut in_quotes = false;
            let mut in_brackets = 0;
            let mut in_parentheses = 0;

            for c in expr.chars() {
                match c {
                    '"' | '\'' if !in_quotes => {
                        in_quotes = true;
                        buffer.push('"');
                    }
                    '"' | '\'' if in_quotes => {
                        in_quotes = false;
                        buffer.push('"');
                    }
                    '(' if !in_quotes => {
                        in_brackets += 1;
                        buffer.push('(');
                    }
                    ')' if !in_quotes => {
                        in_brackets -= 1;
                        buffer.push(')');
                    }
                    '[' if !in_quotes => {
                        in_parentheses += 1;
                        buffer.push('[');
                    }
                    ']' if !in_quotes => {
                        in_parentheses -= 1;
                        buffer.push(']');
                    }
                    ' ' | '　' if !in_quotes && in_brackets == 0 && in_parentheses == 0 => {
                        if !buffer.is_empty() {
                            elements.push(buffer.clone());
                            buffer.clear();
                        }
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
        }

        let tokens: Vec<String> = tokenize_expression(&expr);
        let mut stack: Vec<Type> = Vec::new();
        self.log_print(format!("+-- 式を計算します"));
        for item in tokens {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            if let ExecutionMode::Script = self.execution_mode {
            } else {
                // スタック内部を表示
                println!(
                    "| Stack〔{} 〕←  {}",
                    stack
                        .iter()
                        .map(|x| self.type_string(x.clone()))
                        .collect::<Vec<String>>()
                        .join(", "),
                    item
                );
            }

            // 関数を呼び出す
            if item == "()" {
                let func = match stack.pop() {
                    Some(i) => i,
                    None => return ReturnValue::Error("エラー! スタックが空です".to_string()),
                };

                // 標準ライブラリ関数呼び出し
                let value = if let Type::String(name) = func.clone() {
                    let mut args: Vec<Type> = Vec::new();
                    let count = self.std_func_args_len(name.clone());

                    for _ in 0..count {
                        args.push(stack.pop().unwrap());
                    }
                    self.call_stdlib(name, args)
                } else {
                    // ユーザ定義関数呼び出し
                    if let Type::Function(pointer) = func {
                        let mut args: Vec<Type> = Vec::new();
                        let count = self.function_propaty(pointer).clone().args.len();

                        for _ in 0..count {
                            args.push(stack.pop().unwrap());
                        }

                        self.execute_function(pointer, args)
                    } else {
                        return ReturnValue::Error(
                            "エラー! 関数オブジェクトのみ呼び出し可能です".to_string(),
                        );
                    }
                };

                stack.push(match value {
                    ReturnValue::Some(i) => i,
                    ReturnValue::Error(e) => return ReturnValue::Error(e),
                    _ => Type::Number(0.0),
                });
                continue;
            }

            if let Some(variable) = self.get_variable(item.to_string()) {
                if let Type::Function(pointer) = variable.value {
                    stack.push(Type::Function(pointer));
                    continue;
                }
            }

            match item.parse::<f64>() {
                Ok(i) => stack.push(Type::Number(i)),
                Err(_) => {
                    if self.reference_variable(item.to_string()).is_some() {
                        stack.push(self.get_variable_value(item.to_string()));
                    } else {
                        if stack.len() >= 2 {
                            let y = match stack.pop() {
                                Some(i) => i,
                                None => Type::Number(0.0),
                            };

                            let x = match stack.pop() {
                                Some(i) => i,
                                None => Type::Number(0.0),
                            };

                            // リストの値を得る
                            if item.contains("[") && item.contains("]") {
                                stack.push(x);
                                stack.push(
                                    match self.get_sequence_value(
                                        y,
                                        item[..item.len() - 1]
                                            .to_string()
                                            .replacen("[", "", 1)
                                            .to_string(),
                                    ) {
                                        ReturnValue::Some(i) => i,
                                        ReturnValue::Error(e) => return ReturnValue::Error(e),
                                        _ => Type::Number(0.0),
                                    },
                                );
                                continue;
                            }

                            match (y.clone(), x.clone()) {
                                (Type::String(y), Type::String(x)) => {
                                    match item {
                                        "+" => stack.push(Type::String(x + &y)),
                                        "=" => stack.push(Type::Bool(x == y)),
                                        ">" => stack.push(Type::Bool(x > y)),
                                        "<" => stack.push(Type::Bool(x < y)),
                                        _ => {
                                            stack.push(Type::String(x));
                                            stack.push(Type::String(y));
                                            // 文字列として処理する
                                            if item == "true" {
                                                stack.push(Type::Bool(true));
                                            } else if item == "false" {
                                                stack.push(Type::Bool(false));
                                            } else {
                                                stack.push(Type::String(
                                                    item.replace("'", "")
                                                        .replace('"', "")
                                                        .to_string(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                (Type::Bool(y), Type::Bool(x)) => {
                                    match item {
                                        "&" => stack.push(Type::Bool(x && y)),
                                        "=" => stack.push(Type::Bool(x == y)),
                                        "|" => stack.push(Type::Bool(x || y)),
                                        "!" => {
                                            stack.push(Type::Bool(x));
                                            stack.push(Type::Bool(!y));
                                            continue;
                                        }
                                        _ => {
                                            stack.push(Type::Bool(x));
                                            stack.push(Type::Bool(y));
                                            // 文字列として処理する
                                            if item == "true" {
                                                stack.push(Type::Bool(true));
                                            } else if item == "false" {
                                                stack.push(Type::Bool(false));
                                            } else {
                                                stack.push(Type::String(
                                                    item.replace("'", "")
                                                        .replace('"', "")
                                                        .to_string(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                (Type::Number(y), Type::Number(x)) => {
                                    match item {
                                        "+" => stack.push(Type::Number(x + y)),
                                        "-" => stack.push(Type::Number(x - y)),
                                        "*" => stack.push(Type::Number(x * y)),
                                        "/" => stack.push(Type::Number(x / y)),
                                        "%" => stack.push(Type::Number(x % y)),
                                        "^" => stack.push(Type::Number(x.powf(y))),
                                        "=" => stack.push(Type::Bool(x == y)),
                                        ">" => stack.push(Type::Bool(x > y)),
                                        "<" => stack.push(Type::Bool(x < y)),
                                        "~" => {
                                            stack.push(Type::Number(x));

                                            stack.push(
                                                match self.access(Type::String(y.to_string())) {
                                                    ReturnValue::Some(i) => i,
                                                    ReturnValue::Error(e) => {
                                                        return ReturnValue::Error(e)
                                                    }
                                                    _ => Type::Number(0.0),
                                                },
                                            );
                                            continue;
                                        }
                                        _ => {
                                            stack.push(Type::Number(x));
                                            stack.push(Type::Number(y));
                                            // 文字列として処理する
                                            if item == "true" {
                                                stack.push(Type::Bool(true));
                                            } else if item == "false" {
                                                stack.push(Type::Bool(false));
                                            } else {
                                                stack.push(Type::String(
                                                    item.replace("'", "")
                                                        .replace('"', "")
                                                        .to_string(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                (y, Type::List(mut x)) => {
                                    match item {
                                        "+" => {
                                            x.append(&mut vec![y]);
                                            stack.push(Type::List(x.to_owned()));
                                        }
                                        _ => {
                                            stack.push(Type::List(x));
                                            stack.push(y);
                                            // 文字列として処理する
                                            if item == "true" {
                                                stack.push(Type::Bool(true));
                                            } else if item == "false" {
                                                stack.push(Type::Bool(false));
                                            } else {
                                                stack.push(Type::String(
                                                    item.replace("'", "")
                                                        .replace('"', "")
                                                        .to_string(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    self.log_print("エラー!型が一致しません".to_string());
                                    return ReturnValue::Error(
                                        "エラー!型が一致しません".to_string(),
                                    );
                                }
                            }
                        } else if stack.len() == 1 {
                            // オペランドが一つの演算子
                            let y = match stack.pop() {
                                Some(i) => i,
                                None => Type::Number(0.0),
                            };

                            // リストの値を得る
                            if item.contains("[") && item.contains("]") {
                                stack.push(
                                    match self.get_sequence_value(
                                        y,
                                        item[..item.len() - 1]
                                            .to_string()
                                            .replacen("[", "", 1)
                                            .to_string(),
                                    ) {
                                        ReturnValue::Some(i) => i,
                                        ReturnValue::Error(e) => return ReturnValue::Error(e),
                                        _ => Type::Number(0.0),
                                    },
                                );
                                continue;
                            }

                            match item {
                                "!" => {
                                    let y = match y {
                                        Type::Bool(b) => b,
                                        _ => {
                                            println!("型が一致しません");
                                            false
                                        }
                                    };
                                    stack.push(Type::Bool(!y));
                                    continue;
                                }
                                "~" => {
                                    let y = match y {
                                        Type::Number(i) => i,
                                        _ => {
                                            println!("型が一致しません");
                                            0.0
                                        }
                                    };

                                    stack.push(match self.access(Type::String(y.to_string())) {
                                        ReturnValue::Some(i) => i,
                                        ReturnValue::Error(e) => return ReturnValue::Error(e),
                                        _ => Type::Number(0.0),
                                    });
                                    continue;
                                }

                                _ => {
                                    stack.push(y);
                                    // 文字列として処理する
                                    if item == "true" {
                                        stack.push(Type::Bool(true));
                                    } else if item == "false" {
                                        stack.push(Type::Bool(false));
                                    } else {
                                        stack.push(Type::String(
                                            item.replace("'", "").replace('"', "").to_string(),
                                        ));
                                    }
                                }
                            }
                        } else {
                            // 文字列として処理する
                            if item == "true" {
                                stack.push(Type::Bool(true));
                            } else if item == "false" {
                                stack.push(Type::Bool(false));
                            } else {
                                stack.push(Type::String(
                                    item.replace("'", "").replace('"', "").to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        let result = stack.pop().unwrap_or(Type::Number(0.0));
        let value = self.type_string(result.clone());
        self.log_print(format!("結果 = {}", value));

        return ReturnValue::Some(result);
    }
}
