# Research and constraints

Upstream Forui: https://github.com/duobaseio/forui, revision eea1e8e651d263f14ef43897ccbb7a2576549da5. The upstream root and package licenses contain MIT plus SIL OFL; the package separately credits Inter. `forui_lucide/LICENSE` contains separate notices. Asset-by-asset license analysis remains open; no upstream assets are included here.

Harness: https://github.com/walkinglabs/learn-harness-engineering, revision a9342ff471f2c0e60be0c08b9f4a7c502df4e442. Its harness-creator identifies five subsystems: instructions, state, verification, scope and lifecycle. Its structural assessment does not demonstrate real agent reliability. The local implementation is independent; no upstream script has been executed.

Agent Skills: https://github.com/anthropics/skills refers to https://agentskills.io/specification. Two project skills use frontmatter and were checked with the local skill-creator validator. Their availability in other runtimes and actual activation must still be checked on session startup.

All eight named external repositories resolved a HEAD via `git ls-remote`, but HEAD availability is not a script audit or installation. None of their third-party scripts was installed or executed. Skill names, licenses, dependencies and safety still require a pinned, file-by-file audit. egui-agent-skills publicly lists `egui`, `egui-review`, `egui-kittest`, `eframe-project`, `egui-ui-design` and `egui-profiler`; its version targeting must be checked against the actual egui version selected.

CodeQL documents Rust support: https://codeql.github.com/docs/codeql-overview/supported-languages-and-frameworks/. This is not evidence that a scan ran. OCR's configuration, model access, terms, and security properties have not been verified. AI review may incur cost and requires an authorized service; absence is a blocker, not a pass.

Initial name screening rejected Velori (https://velori.dev/) and Nuvori (https://nuvori.cloud/) as existing software brands. This is not a trademark search. No project name is selected; registry and relevant jurisdiction trademark checks remain open.

Local versions: Flutter 3.47.7 and Dart 3.13.5 satisfy the version ranges declared in the upstream pubspec; analyzer dependency resolution and actual analysis have not been run. The checker identified gaps in gate classification, task closure, Gitleaks history scanning and command-output handling. Two reproduced runner flaws (empty profiles and overbroad evidence exclusions) were corrected. The runner records command output hashes rather than persisting future raw output to reduce secret exposure, but it is not a secure attestation service. Previous logs contain only outputs from the commands actually run at this stage.

Loop engineering suggests generator/evaluator separation and persistent state; graph engineering introduces explicit nodes, routing and shared state where useful. This workspace has an independent review and state files but does not implement an autonomous loop, graph scheduler or bounded repair cycle. Those remain to be built and tested.
