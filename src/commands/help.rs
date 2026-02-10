use colored::Colorize;

pub fn execute() {
    println!("{}", "Process CLI — Decision Pipeline Engine".bold());
    println!("{}", "━".repeat(45));

    print_section("Phase 0: Initialization", &[
        ("init", "Initialize project and process"),
        ("status", "Show current state, progress, branches"),
        ("seed-validate", "Validate seed.yaml 6-field spec"),
        ("ai-config show|test|set-provider", "AI provider management"),
    ]);

    print_section("Phase 1: Diverge", &[
        ("diverge", "Generate ≥2 architectural proposals"),
        ("diverge-validate", "Validate proposal format"),
        ("diverge-challenge", "Challenge proposals (decision depth)"),
    ]);

    print_section("Phase 2: Converge", &[
        ("converge", "Analyze proposals, extract rules, choose one"),
        ("converge-validate", "Validate rules format"),
        ("converge-challenge", "Challenge chosen approach"),
    ]);

    print_section("Phase 3: Skeleton", &[
        ("skeleton", "Generate project structure"),
        ("skeleton-validate", "Validate skeleton output"),
    ]);

    print_section("Phase 4: Branch Loop", &[
        ("branch new <name>", "Create branch hypothesis"),
        ("branch start <name>", "Validate and create git branch"),
        ("branch review <name> [-r role]", "Multi-role AI review"),
        ("branch abuse <name>", "Adversarial testing"),
        ("branch gate <name>", "Merge gate checks"),
        ("branch merge <name>", "Mark branch merged"),
    ]);

    print_section("Phase 5-7: Finalization", &[
        ("stabilize", "Freeze invariants"),
        ("postmortem", "AI-generated decision retrospective"),
        ("done", "Mark project complete"),
    ]);

    print_section("Adopt (existing projects)", &[
        ("adopt scan-structure", "Detect language/framework/dirs"),
        ("adopt scan-dependencies", "Parse dependency manifests"),
        ("adopt infer-conventions", "Infer coding conventions (AI)"),
        ("adopt scan-git-history", "Extract decisions from git (AI)"),
        ("adopt gap-analysis", "Identify missing decisions (AI)"),
        ("adopt all", "Run all adopt passes"),
    ]);

    print_section("Automation", &[
        ("generate git-hooks|cicd|makefile|ide|all", "Generate project files"),
        ("check sensitive|todo|lint|test|all", "Run automated checks"),
    ]);

    print_section("Pass Engine", &[
        ("pass run <name>", "Run a specific pass"),
        ("pass list", "List all available passes"),
        ("pass run-all", "Run all passes in dependency order"),
    ]);

    print_section("Utilities", &[
        ("learn <lesson> [-c category]", "Record a learning"),
        ("friction <branch> <desc> [-s severity]", "Record friction point"),
        ("completions bash|zsh|fish", "Generate shell completions"),
    ]);

    println!("\n{}", "Use `process <command> --help` for detailed usage.".dimmed());
}

fn print_section(title: &str, commands: &[(&str, &str)]) {
    println!("\n  {}", title.bold().cyan());
    for (cmd, desc) in commands {
        println!("    {:<42} {}", cmd.green(), desc);
    }
}
