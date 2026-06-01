use swc_ecma_ast::{Expr, Lit, ModuleDecl, ModuleItem, Program, Stmt};

use crate::parser::parse_program;

pub fn transform_domain_lock(program: &mut Program, domains: &[String], redirect_url: &str) {
    if domains.is_empty() {
        return;
    }

    let normalized_domains = normalize_domains(domains);
    if normalized_domains.is_empty() {
        return;
    }

    insert_domain_lock_helper(
        program,
        create_domain_lock_statements(&normalized_domains, redirect_url),
    );
}

fn normalize_domains(domains: &[String]) -> Vec<String> {
    domains
        .iter()
        .map(|domain| extract_domain_from(domain).to_lowercase())
        .filter(|domain| !domain.is_empty())
        .collect()
}

fn extract_domain_from(value: &str) -> String {
    let host = if value.contains("://") || value.starts_with("//") {
        value.split('/').nth(2).unwrap_or("")
    } else {
        value.split('/').next().unwrap_or("")
    };

    host.split(':').next().unwrap_or("").to_string()
}

fn insert_domain_lock_helper(program: &mut Program, statements: Vec<Stmt>) {
    match program {
        Program::Script(script) => {
            let insert_index = first_non_directive_statement_index(&script.body);

            script.body.splice(insert_index..insert_index, statements);
        }
        Program::Module(module) => {
            let first_non_import_index = module
                .body
                .iter()
                .position(|item| !matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(_))))
                .unwrap_or(module.body.len());
            let directive_count = module.body[first_non_import_index..]
                .iter()
                .take_while(|item| {
                    matches!(item, ModuleItem::Stmt(statement) if is_directive_statement(statement))
                })
                .count();
            let insert_index = first_non_import_index + directive_count;

            module.body.splice(
                insert_index..insert_index,
                statements.into_iter().map(ModuleItem::Stmt),
            );
        }
    }
}

fn first_non_directive_statement_index(statements: &[Stmt]) -> usize {
    statements
        .iter()
        .position(|statement| !is_directive_statement(statement))
        .unwrap_or(statements.len())
}

fn is_directive_statement(statement: &Stmt) -> bool {
    matches!(
        statement,
        Stmt::Expr(expression_statement)
            if matches!(expression_statement.expr.as_ref(), Expr::Lit(Lit::Str(_)))
    )
}

fn create_domain_lock_statements(domains: &[String], redirect_url: &str) -> Vec<Stmt> {
    let domains_json = escape_json_slashes(
        serde_json::to_string(domains).expect("normalized domain lock domains should serialize"),
    );
    let redirect_expression = string_from_char_codes_expression(redirect_url);
    let helper_source = format!(
        r#"
            (function () {{
                const domainLockDomains = {domains_json};
                const domainLockRedirectUrl = {redirect_expression};
                const domainLockGlobal = typeof globalThis !== 'undefined'
                    ? globalThis
                    : typeof self !== 'undefined'
                        ? self
                        : typeof window !== 'undefined'
                            ? window
                            : typeof global !== 'undefined'
                                ? global
                                : this;
                const domainLockDocument = domainLockGlobal.document;

                if (!domainLockDocument) {{
                    return;
                }}

                const domainLockLocation = domainLockDocument.location;
                const domainLockCurrentDomain = String(
                    domainLockDocument.domain ||
                    (domainLockLocation && domainLockLocation.hostname) ||
                    ''
                ).toLowerCase();

                if (!domainLockCurrentDomain) {{
                    return;
                }}

                let domainLockAllowed = false;

                for (let domainLockIndex = 0; domainLockIndex < domainLockDomains.length; domainLockIndex++) {{
                    const domainLockDomain = domainLockDomains[domainLockIndex];
                    const domainLockNormalized = domainLockDomain[0] === '.'
                        ? domainLockDomain.slice(1)
                        : domainLockDomain;
                    const domainLockPosition = domainLockCurrentDomain.length - domainLockNormalized.length;
                    const domainLockLastIndex = domainLockCurrentDomain.indexOf(
                        domainLockNormalized,
                        domainLockPosition
                    );
                    const domainLockEndsWith = domainLockLastIndex !== -1 &&
                        domainLockLastIndex === domainLockPosition;

                    if (domainLockEndsWith) {{
                        if (domainLockCurrentDomain.length === domainLockDomain.length ||
                            domainLockDomain.indexOf('.') === 0) {{
                            domainLockAllowed = true;
                        }}
                    }}
                }}

                if (!domainLockAllowed) {{
                    domainLockDocument.location = domainLockRedirectUrl;
                }}
            }})();
        "#
    );
    let parsed_program = parse_program(&helper_source).expect("domain lock helper should parse");

    match parsed_program.program {
        Program::Script(script) => script.body,
        Program::Module(module) => module
            .body
            .into_iter()
            .filter_map(|item| match item {
                ModuleItem::Stmt(statement) => Some(statement),
                ModuleItem::ModuleDecl(_) => None,
            })
            .collect(),
    }
}

fn escape_json_slashes(value: String) -> String {
    value.replace('/', "\\/")
}

fn string_from_char_codes_expression(value: &str) -> String {
    let char_codes = value
        .chars()
        .map(|character| u32::from(character).to_string())
        .collect::<Vec<_>>()
        .join(",");

    format!("String.fromCharCode({char_codes})")
}

#[cfg(test)]
mod tests {
    use crate::codegen::generate_code;
    use crate::parser::parse_program;

    use super::*;

    fn transform(source_code: &str, domains: &[String], redirect_url: &str) -> String {
        let mut parsed_program = parse_program(source_code).expect("source should parse");

        transform_domain_lock(&mut parsed_program.program, domains, redirect_url);

        generate_code(&parsed_program.program, parsed_program.source_map, true)
            .expect("transformed code should generate")
    }

    #[test]
    fn prepends_domain_lock_helper_when_domains_are_configured() {
        let code = transform(
            "globalThis.result = 'ran';",
            &[".example.com".to_string()],
            "about:blank",
        );

        assert!(
            code.find("domainLockDomains")
                .expect("helper should be present")
                < code
                    .find("globalThis.result")
                    .expect("user code should be present"),
            "{code}"
        );
    }

    #[test]
    fn skips_domain_lock_helper_when_domains_are_empty() {
        let code = transform("globalThis.result = 'ran';", &[], "about:blank");

        assert_eq!(code, "globalThis.result='ran';");
    }

    #[test]
    fn normalizes_configured_domains() {
        let domains = normalize_domains(&[
            "https://Example.com:9000/path".to_string(),
            "//Sub.Example.com:443/abc".to_string(),
        ]);

        assert_eq!(domains, vec!["example.com", "sub.example.com"]);
    }
}
