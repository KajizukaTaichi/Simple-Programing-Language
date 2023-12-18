use crate::executor::{ControlMode, ExecutionMode, Executor};

impl<'a> Executor<'a> {
    /// 構文チェック
    pub fn check(&mut self, code: Vec<String>) {
        for code in code {
            let code = if let ControlMode::Function = self.control_mode {
                code.as_str()
            } else {
                code.trim().split("#").collect::<Vec<&str>>()[0]
            };
            if code.is_empty() {
                continue;
            }

            match self.control_mode {
                ControlMode::For => {
                    if code.contains("end for") || code.contains("endfor") {
                        // ネストの階層を判別する
                        if self.nest_for > 0 {
                            self.nest_for -= 1;
                            self.stmt.push(code.to_string());
                        } else {
                            self.control_mode = ControlMode::Normal;
                            Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .check(self.stmt.clone());
                            self.stmt = Vec::new();
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
                            self.control_mode = ControlMode::Normal;
                            Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .check(self.stmt.clone());
                            self.stmt = Vec::new();
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
                            self.control_mode = ControlMode::Normal;
                            Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .check(self.else_stmt.clone());

                            self.else_stmt = Vec::new();
                            self.stmt = Vec::new();
                            Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .check(self.stmt.clone());
                            self.else_stmt = Vec::new();
                            self.stmt = Vec::new();
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
                            self.control_mode = ControlMode::Normal;
                            Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .check(self.stmt.clone());
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
                            self.control_mode = ControlMode::Normal;
                            Executor::new(
                                &mut self.memory,
                                &mut self.name_space,
                                self.execution_mode.clone(),
                            )
                            .check(self.stmt.clone());
                            self.stmt = Vec::new();
                        }
                    } else if code.contains("func") {
                        self.nest_func += 1;

                        self.stmt.push(code.to_string());
                    } else {
                        self.stmt.push(code.to_string());
                    }
                }

                ControlMode::Normal => {
                    if code.contains("var") {
                        // 変数の定義
                        let new_code = code.replacen("var", "", 1);
                        if !&new_code.contains("=") {
                            println!("エラー! 変数名と式の間にイコールをいれてください");
                        }
                    } else if code.contains("func") {
                        //　関数の定義
                        if !code.contains("(") || !code.contains(")") {
                            println!("エラー! 関数にはカッコをつけてください");
                        }
                        self.control_mode = ControlMode::Function;
                    } else if code.contains("call") {
                        // 関数呼び出し
                        if !code.contains("(") || !code.contains(")") {
                            println!("エラー! 関数にはカッコをつけてください");
                        }
                    } else if code.contains("for") {
                        self.control_mode = ControlMode::For;
                    } else if code.contains("if") {
                        self.control_mode = ControlMode::If
                    } else if code.contains("while") {
                        self.control_mode = ControlMode::While;
                    } else {
                        if let ExecutionMode::Script = self.execution_mode {
                        } else {
                            println!("エラー! コマンドが不正です: {}", code);
                        }
                    }
                }
            }
        }
        match self.control_mode {
            ControlMode::Function => {
                println!("エラー! 関数の終わりが見つかりません");
            }
            ControlMode::If | ControlMode::Else => {
                println!("エラー! if文の終わりが見つかりません");
            }
            ControlMode::For => {
                println!("エラー! for文の終わりが見つかりません");
            }
            ControlMode::While => {
                println!("エラー! while文の終わりが見つかりません");
            }
            ControlMode::Normal => {}
        };
    }
}
