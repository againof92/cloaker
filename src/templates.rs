use crate::engine::helpers::html_escape;
use crate::models::RedirectLink;

// ============================
// SIDEBAR reutilizável
// ============================
fn sidebar(active: &str) -> String {
    let items = [
        ("Dashboard", "/m4ciel7"),
        ("Links", "/m4ciel7/links"),
        ("Logs", "/m4ciel7/logs"),
        ("Criar Link", "/m4ciel7/create"),
        ("Configuracoes", "/m4ciel7/config"),
    ];
    let mut s = String::from(r#"<div class="sidebar"><div class="logo">M4CIEL</div><div class="menu">"#);
    for (label, href) in &items {
        let cls = if *label == active { " class=\"active\"" } else { "" };
        s.push_str(&format!(r#"<a{} href="{}">{}</a>"#, cls, href, label));
    }
    s.push_str("</div></div>");
    s
}

fn base_css() -> &'static str {
    r#"*{box-sizing:border-box;margin:0;padding:0}
body{font-family:'Inter',-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0f0f0f;color:#e6e6e6;font-size:14px}
.sidebar{position:fixed;left:0;top:0;width:220px;height:100vh;background:#151515;padding:20px;border-right:1px solid #2a2a2a}
.logo{font-size:20px;font-weight:700;color:#9bbcff;margin-bottom:24px}
.menu a{display:block;color:#9aa;text-decoration:none;padding:10px 12px;border-radius:8px;margin-bottom:6px}
.menu a:hover,.menu a.active{background:#242424;color:#fff}
.main{margin-left:220px;padding:24px}
.card{background:#151515;border:1px solid #2a2a2a;border-radius:12px;padding:18px;margin-bottom:18px}
.card h2{font-size:16px;color:#9bbcff;margin-bottom:12px}
.btn{background:#3a3f50;border:1px solid #2a2a2a;color:#fff;padding:6px 10px;border-radius:6px;text-decoration:none;cursor:pointer;font-size:12px}
.btn:hover{background:#4a5166}
.btn.primary{background:#667eea;border-color:#667eea}
.btn.primary:hover{background:#5a6fd6}
.btn.danger{background:#7b2e2e;border-color:#7b2e2e}
.btn.danger:hover{background:#8b3535}
.form-group{margin-bottom:14px}
label{display:block;font-size:13px;color:#9aa;margin-bottom:6px}
input[type="text"],input[type="url"],input[type="number"],input[type="password"],textarea{width:100%;padding:10px 12px;background:#1f1f1f;border:1px solid #2a2a2a;border-radius:8px;color:#fff;font-size:14px}
textarea{resize:vertical}
.grid-2{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px}
.rules{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:10px}
.rule{background:#1f1f1f;border:1px solid #2d2d2d;border-radius:10px;padding:10px;font-size:13px;color:#ddd}
.hint{font-size:12px;color:#777;margin-top:6px}
.table{width:100%;border-collapse:collapse}
.table th,.table td{padding:10px;border-bottom:1px solid #2a2a2a;font-size:13px;text-align:left;vertical-align:top}
.table th{color:#9aa;font-weight:600}
.badge{display:inline-block;padding:4px 8px;border-radius:6px;font-size:12px}
.badge.active{background:#235b3d;color:#a7f3c1}
.badge.paused{background:#5a2d2d;color:#f3a7a7}
.badge-success{background:#235b3d;color:#a7f3c1}
.badge-danger{background:#5a2d2d;color:#f3a7a7}
.badge-warning{background:#5a4a2d;color:#f3e3a7}
code{background:#111;border:1px solid #2a2a2a;padding:4px 6px;border-radius:6px;font-size:12px;color:#ddd;display:inline-block}
.access{display:flex;gap:8px;align-items:center;flex-wrap:wrap}
.stats{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px;margin-bottom:18px}
.stat{background:#151515;border:1px solid #2a2a2a;border-radius:12px;padding:16px}
.stat-value{font-size:24px;font-weight:700;color:#9bbcff}
.stat-label{font-size:12px;color:#9aa;margin-top:4px}
.checkbox{display:flex;align-items:center;gap:8px;color:#ddd;font-size:14px}
@media(max-width:900px){.sidebar{position:static;width:100%;height:auto;border-right:none;border-bottom:1px solid #2a2a2a}.main{margin-left:0}.stats{grid-template-columns:repeat(2,minmax(0,1fr))}.rules,.grid-2{grid-template-columns:1fr}.table{display:block;overflow-x:auto;white-space:nowrap}}
@media(max-width:600px){.stats{grid-template-columns:1fr}}"#
}

fn head(title: &str, extra_css: &str) -> String {
    format!(
        r#"<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{}</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
<style>{}{}</style></head><body>"#,
        html_escape(title), base_css(), extra_css
    )
}

// ============================
// SAFE PAGE
// ============================
pub fn safe_page() -> String {
    r#"<!DOCTYPE html><html lang="pt-BR"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>Bem-vindo</title>
<style>*{margin:0;padding:0;box-sizing:border-box}body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#000;color:#fff;min-height:100vh;display:flex;align-items:center;justify-content:center}.container{text-align:center;padding:40px}.box{background:#1a1a1a;border-radius:16px;padding:40px 60px;box-shadow:0 10px 40px rgba(0,0,0,0.5)}.code{font-size:80px;font-weight:bold;color:#fff;background:#2a2a2a;border-radius:8px;padding:15px 40px;display:inline-block;margin-bottom:25px}h1{font-size:24px;font-weight:600;margin-bottom:10px}p{color:#999;font-size:16px;margin-bottom:30px}.btn{display:inline-block;background:#fff;color:#000;padding:14px 40px;border-radius:8px;text-decoration:none;font-weight:600;font-size:16px;transition:all .2s}.btn:hover{background:#f0f0f0;transform:translateY(-2px)}@media(max-width:900px){.container{padding:20px}.box{padding:28px 24px}.code{font-size:56px;padding:12px 28px}h1{font-size:20px}p{font-size:14px}.btn{width:100%;padding:12px 20px}}</style></head><body><div class="container"><div class="box"><div class="code">200</div><h1>Bem-vindo ao nosso site</h1><p>Descubra ofertas e conteudos exclusivos.</p><a href="/" class="btn">Voltar ao inicio</a></div></div></body></html>"#.to_string()
}

// ============================
// LOGIN PAGE
// ============================
pub fn login_page(error_msg: &str) -> String {
    let error_html = if error_msg.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="error">{}</div>"#, html_escape(error_msg))
    };
    format!(r#"<!DOCTYPE html><html lang="pt-BR"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>404 Not Found</title><meta name="robots" content="noindex,nofollow">
<style>*{{margin:0;padding:0;box-sizing:border-box}}body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0a0a0a;color:#fff;min-height:100vh;display:flex;align-items:center;justify-content:center}}.login-box{{background:#1a1a1a;padding:40px;border-radius:16px;width:100%;max-width:400px;border:1px solid #333;display:none}}.logo{{text-align:center;font-size:28px;font-weight:bold;margin-bottom:30px;color:#667eea}}.form-group{{margin-bottom:20px}}label{{display:block;margin-bottom:8px;color:#888;font-size:14px}}input{{width:100%;padding:14px;border-radius:8px;border:1px solid #333;background:#252525;color:#fff;font-size:16px}}input:focus{{outline:none;border-color:#667eea}}button{{width:100%;padding:14px;border-radius:8px;border:none;background:linear-gradient(135deg,#667eea 0%,#764ba2 100%);color:#fff;font-size:16px;font-weight:600;cursor:pointer;transition:transform .2s}}button:hover{{transform:translateY(-2px)}}.error{{background:#ff4444;color:#fff;padding:12px;border-radius:8px;margin-bottom:20px;text-align:center;font-size:14px}}.fake-404{{text-align:center;color:#666}}@media(max-width:900px){{body{{padding:20px}}.login-box{{padding:24px;max-width:100%}}.logo{{font-size:24px;margin-bottom:20px}}}}</style></head><body>
<div class="fake-404" id="fake">404 page not found</div>
<div class="login-box" id="real">
<div class="logo">👤 M4CIEL</div>
{error_html}
<form method="POST" id="loginForm">
<div class="form-group"><label>Usuario</label><input type="text" name="username" placeholder="Digite seu usuario" required></div>
<div class="form-group"><label>Senha</label><input type="password" name="password" placeholder="Digite sua senha" required></div>
<input type="hidden" name="js_check" id="jsCheck" value="">
<button type="submit">Entrar</button>
</form></div>
<script>
(function(){{
var isHuman=true;
if(typeof window==='undefined'||typeof document==='undefined')isHuman=false;
if(navigator.webdriver===true)isHuman=false;
if(navigator.plugins.length===0&&!/mobile|android|iphone/i.test(navigator.userAgent))isHuman=false;
if(!navigator.languages||navigator.languages.length===0)isHuman=false;
if(window._phantom||window.phantom)isHuman=false;
if(window.__nightmare)isHuman=false;
if(screen.width===0||screen.height===0)isHuman=false;
if(isHuman){{
document.getElementById('fake').style.display='none';
document.getElementById('real').style.display='block';
document.getElementById('jsCheck').value=btoa(Date.now().toString());
document.querySelector('input[name="username"]').focus();
}}
}})();
</script></body></html>"#)
}

// ============================
// DASHBOARD
// ============================
pub fn dashboard_page(
    total_links: i32, total_clicks: i32, total_blocked: i32, block_rate: i32,
    param_name: &str, fb_rule: &str, map_svg: &str, state_counts_json: &str, state_names_json: &str,
) -> String {
    let extra_css = r#".map-wrap{display:grid;gap:12px}.map-container{background:#111;border:1px solid #2a2a2a;border-radius:12px;padding:12px}#br-map svg{width:100%;height:auto;display:block}.map-legend{display:flex;gap:10px;align-items:center;font-size:12px;color:#9aa;flex-wrap:wrap}.legend-swatch{width:14px;height:14px;border-radius:4px;background:#1f1f1f;border:1px solid #2a2a2a}.state-name{font-size:8px;fill:#cfd6ff;font-weight:600;pointer-events:none;paint-order:stroke;stroke:#0f0f0f;stroke-width:1}.state-count{font-size:10px;fill:#ffffff;font-weight:700;pointer-events:none;paint-order:stroke;stroke:#0f0f0f;stroke-width:1}"#;
    let map = if map_svg.trim().is_empty() { "<div class=\"hint\">Mapa indisponivel.</div>" } else { map_svg };
    format!(r#"{head}{sidebar}
<div class="main">
<h1 style="margin-bottom:16px">Dashboard</h1>
<div class="stats">
<div class="stat"><div class="stat-value" id="total-links">{total_links}</div><div class="stat-label">Links ativos</div></div>
<div class="stat"><div class="stat-value" id="total-clicks">{total_clicks}</div><div class="stat-label">Cliques</div></div>
<div class="stat"><div class="stat-value" id="total-blocked">{total_blocked}</div><div class="stat-label">Bloqueados</div></div>
<div class="stat"><div class="stat-value" id="block-rate">{block_rate}%</div><div class="stat-label">Taxa de bloqueio</div></div>
</div>
<div class="card"><h2 style="margin-bottom:12px;color:#9bbcff;font-size:16px">Regras fixas (sempre ativas)</h2>
<div class="rules">
<div class="rule">Parametro secreto obrigatorio (?{param_name}=...)</div>
<div class="rule">{fb_rule}</div>
<div class="rule">Somente mobile iPhone/Android</div>
<div class="rule">Bloqueio de bots e automacao</div>
</div></div>
<div class="card"><h2 style="margin-bottom:12px;color:#9bbcff;font-size:16px">Usuarios por estado (BR)</h2>
<div class="map-wrap">
<div class="map-container" id="br-map">{map}</div>
<div class="map-legend"><span class="legend-swatch"></span><span>Usuarios reais (permitidos) - IP unico por dia</span></div>
</div></div></div>
<script>
let stateCounts={state_counts_json};const stateNames={state_names_json};
function colorFor(c,m){{if(!m||c<=0)return'#1b1b1b';const t=Math.min(c/m,1);return'rgb('+Math.round(27+(102-27)*t)+','+Math.round(27+(126-27)*t)+','+Math.round(27+(234-27)*t)+')';}}
function renderMap(){{const c=document.getElementById('br-map');if(!c)return;const s=c.querySelector('svg');if(!s)return;const e=s.querySelector('#state-labels');if(e)e.remove();const v=Object.values(stateCounts||{{}});const mx=v.length?Math.max(...v):0;const ns='http://www.w3.org/2000/svg';const g=document.createElementNS(ns,'g');g.setAttribute('id','state-labels');s.appendChild(g);s.querySelectorAll('path[id^="BR"]').forEach(p=>{{const cd=p.id.replace(/^BR/,'');const ct=stateCounts&&stateCounts[cd]?stateCounts[cd]:0;const nm=stateNames&&stateNames[cd]?stateNames[cd]:cd;p.style.fill=colorFor(ct,mx);p.style.stroke='#2f2f2f';p.style.strokeWidth='0.6';p.style.cursor='pointer';const t=document.createElementNS(ns,'title');t.textContent=nm+': '+ct;p.appendChild(t);const b=p.getBBox();const cx=(b.x+b.width/2).toFixed(2);const cy=b.y+b.height/2;const nt=document.createElementNS(ns,'text');nt.setAttribute('x',cx);nt.setAttribute('y',(cy-8).toFixed(2));nt.setAttribute('text-anchor','middle');nt.setAttribute('class','state-name');nt.textContent=nm;g.appendChild(nt);const ct2=document.createElementNS(ns,'text');ct2.setAttribute('x',cx);ct2.setAttribute('y',(cy+10).toFixed(2));ct2.setAttribute('text-anchor','middle');ct2.setAttribute('dominant-baseline','middle');ct2.setAttribute('class','state-count');ct2.textContent=ct;g.appendChild(ct2);}});}}
async function loadStats(){{try{{const r=await fetch('/m4ciel7/stats');if(!r.ok)return;const d=await r.json();document.getElementById('total-links').textContent=d.total_links||0;document.getElementById('total-clicks').textContent=d.total_clicks||0;document.getElementById('total-blocked').textContent=d.total_blocked||0;const t=(d.total_clicks||0)+(d.total_blocked||0);document.getElementById('block-rate').textContent=(t>0?Math.round((d.total_blocked||0)/t*100):0)+'%';}}catch(e){{}}}}
async function loadMapStats(){{try{{const r=await fetch('/m4ciel7/map-stats');if(!r.ok)return;stateCounts=await r.json();renderMap();}}catch(e){{}}}}
loadStats();loadMapStats();setInterval(loadStats,5000);setInterval(loadMapStats,15000);
const mapEs=new EventSource('/m4ciel7/logs/stream');let mapTimer=null;function sched(){{if(mapTimer)return;mapTimer=setTimeout(function(){{mapTimer=null;loadMapStats();}},800);}}mapEs.addEventListener('log',sched);mapEs.addEventListener('clear',sched);
</script></body></html>"#,
        head = head("Dashboard", extra_css),
        sidebar = sidebar("Dashboard"),
        total_links = total_links,
        total_clicks = total_clicks,
        total_blocked = total_blocked,
        block_rate = block_rate,
        param_name = html_escape(param_name),
        fb_rule = html_escape(fb_rule),
        map = map,
        state_counts_json = state_counts_json,
        state_names_json = state_names_json,
    )
}

// ============================
// LINKS PAGE
// ============================
pub fn links_page(param_name: &str, rows: &[(String, String, String, String, i32, i32, bool)]) -> String {
    let mut table_rows = String::new();
    for (id, slug, param_code, offer_url, clicks, blocked, active) in rows {
        let access_path = format!("/go/{}?{}={}", html_escape(slug), html_escape(param_name), html_escape(param_code));
        let (status_class, status_text) = if *active { ("active", "Ativo") } else { ("paused", "Pausado") };
        table_rows.push_str(&format!(
            r#"<tr><td>{slug}</td><td><div class="access"><code>{access}</code><button class="btn copy-btn" type="button" data-path="{access}">Copiar</button></div></td><td><a href="{url}" target="_blank" rel="noreferrer">{url}</a></td><td>{clicks}</td><td>{blocked}</td><td><span class="badge {sc}">{st}</span></td><td><a class="btn" href="/m4ciel7/edit?id={id}">Editar</a> <form method="POST" action="/m4ciel7/delete?id={id}" style="display:inline" onsubmit="return confirm('Excluir este link?');"><button class="btn danger" type="submit">Excluir</button></form></td></tr>"#,
            slug = html_escape(slug), access = html_escape(&access_path),
            url = html_escape(offer_url), clicks = clicks, blocked = blocked,
            sc = status_class, st = status_text, id = html_escape(id),
        ));
    }
    format!(r#"{head}{sidebar}
<div class="main">
<div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:14px"><h1>Links</h1><a class="btn primary" href="/m4ciel7/create">Criar Link</a></div>
<div class="card"><table class="table"><thead><tr><th>Slug</th><th>Acesso</th><th>Oferta</th><th>Cliques</th><th>Bloqueados</th><th>Status</th><th>Acoes</th></tr></thead><tbody>{rows}</tbody></table></div></div>
<script>
function copyText(t){{if(navigator.clipboard){{navigator.clipboard.writeText(t);return;}}var i=document.createElement('input');i.value=t;document.body.appendChild(i);i.select();document.execCommand('copy');document.body.removeChild(i);}}
document.querySelectorAll('.copy-btn').forEach(function(b){{b.addEventListener('click',function(){{var p=b.getAttribute('data-path')||'';copyText(window.location.origin+p);var o=b.textContent;b.textContent='Copiado';setTimeout(function(){{b.textContent=o;}},1200);}});}});
</script></body></html>"#,
        head = head("Links", ""),
        sidebar = sidebar("Links"),
        rows = table_rows,
    )
}

// ============================
// CREATE LINK
// ============================
pub fn create_link_page(param_name: &str, default_code: &str) -> String {
    format!(r#"{head}{sidebar}
<div class="main" style="max-width:980px">
<h1 style="margin-bottom:16px">Criar Link</h1>
<form method="POST">
<div class="card"><h2>Status</h2><label><input type="checkbox" name="active" checked> Link ativo</label></div>
<div class="card"><h2>Regras fixas (sempre ativas)</h2><div class="rules">
<div class="rule">Parametro secreto obrigatorio (?{pn}=...)</div>
<div class="rule">Somente anuncios Facebook/Instagram</div>
<div class="rule">Somente mobile iPhone/Android</div>
<div class="rule">Bloqueio de bots e automacao</div></div></div>
<div class="card"><h2>Dados do Link</h2>
<div class="grid-2"><div class="form-group"><label>Slug</label><input type="text" name="slug" placeholder="ex: oferta-1" required></div>
<div class="form-group"><label>Parametro secreto</label><input type="text" name="param_code" value="{code}"></div></div>
<div class="form-group"><label>URL da oferta</label><input type="url" name="offer_url" placeholder="https://" required></div>
<div class="form-group"><label>Safe page URL (opcional)</label><input type="url" name="safe_page_url" placeholder="https://"></div>
<div class="hint">Use: /go/slug?{pn}=SEU_CODIGO</div></div>
<div class="card"><h2>Filtros de Localizacao</h2>
<div class="grid-2"><div class="form-group"><label>Paises Permitidos</label><input type="text" name="allowed_countries" placeholder="BR,PT"></div>
<div class="form-group"><label>Paises Bloqueados</label><input type="text" name="blocked_countries" placeholder="US,CA"></div></div>
<div class="grid-2"><div class="form-group"><label>IPs Bloqueados</label><textarea name="blocked_ips" rows="3" placeholder="1.2.3.4"></textarea></div>
<div class="form-group"><label>ISPs Bloqueados</label><textarea name="blocked_isps" rows="3" placeholder="ISP A, ISP B"></textarea></div></div></div>
<div class="card"><h2>Limites e Horarios</h2>
<div class="grid-2"><div class="form-group"><label>Limite de Cliques</label><input type="number" name="max_clicks" value="0" min="0"></div>
<div class="form-group"><label>TTL do Parametro (minutos)</label><input type="number" name="param_ttl" value="0" min="0"></div></div>
<div class="form-group"><label>Horario Permitido</label><input type="text" name="allowed_hours" placeholder="08:00-22:00"></div></div>
<button type="submit" class="btn primary" style="padding:10px 14px;border-radius:8px">Salvar Link</button>
</form></div></body></html>"#,
        head = head("Criar Link", ""),
        sidebar = sidebar("Criar Link"),
        pn = html_escape(param_name),
        code = html_escape(default_code),
    )
}

// ============================
// EDIT LINK
// ============================
pub fn edit_link_page(param_name: &str, link: &RedirectLink) -> String {
    let checked = if link.active { "checked" } else { "" };
    let preview = format!("/go/{}?{}={}", link.slug, param_name, link.param_code);
    format!(r#"{head}{sidebar}
<div class="main" style="max-width:980px">
<h1 style="margin-bottom:16px">Editar Link</h1>
<form method="POST" action="/m4ciel7/edit?id={id}">
<div class="card"><h2>Status</h2><label><input type="checkbox" name="active" {checked}> Link ativo</label></div>
<div class="card"><h2>Regras fixas (sempre ativas)</h2><div class="rules">
<div class="rule">Parametro secreto obrigatorio (?{pn}=...)</div>
<div class="rule">Somente anuncios Facebook/Instagram</div>
<div class="rule">Somente mobile iPhone/Android</div>
<div class="rule">Bloqueio de bots e automacao</div></div></div>
<div class="card"><h2>Dados do Link</h2>
<div class="grid-2"><div class="form-group"><label>Slug</label><input type="text" name="slug" value="{slug}" required></div>
<div class="form-group"><label>Parametro secreto</label><input type="text" name="param_code" value="{pc}"><div class="hint">Preview: <code>{preview}</code></div></div></div>
<div class="form-group"><label>URL da oferta</label><input type="url" name="offer_url" value="{offer}" required></div>
<div class="form-group"><label>Safe page URL (opcional)</label><input type="url" name="safe_page_url" value="{safe}"></div></div>
<div class="card"><h2>Filtros de Localizacao</h2>
<div class="grid-2"><div class="form-group"><label>Paises Permitidos</label><input type="text" name="allowed_countries" value="{ac}"></div>
<div class="form-group"><label>Paises Bloqueados</label><input type="text" name="blocked_countries" value="{bc}"></div></div>
<div class="grid-2"><div class="form-group"><label>IPs Bloqueados</label><textarea name="blocked_ips" rows="3">{bi}</textarea></div>
<div class="form-group"><label>ISPs Bloqueados</label><textarea name="blocked_isps" rows="3">{bis}</textarea></div></div></div>
<div class="card"><h2>Limites e Horarios</h2>
<div class="grid-2"><div class="form-group"><label>Limite de Cliques</label><input type="number" name="max_clicks" value="{mc}" min="0"></div>
<div class="form-group"><label>TTL do Parametro (minutos)</label><input type="number" name="param_ttl" value="{pt}" min="0"></div></div>
<div class="form-group"><label>Horario Permitido</label><input type="text" name="allowed_hours" value="{ah}"></div></div>
<button type="submit" class="btn primary" style="padding:10px 14px;border-radius:8px">Salvar Alteracoes</button>
</form></div></body></html>"#,
        head = head("Editar Link", ""),
        sidebar = sidebar("Links"),
        id = html_escape(&link.id),
        checked = checked,
        pn = html_escape(param_name),
        slug = html_escape(&link.slug),
        pc = html_escape(&link.param_code),
        preview = html_escape(&preview),
        offer = html_escape(&link.offer_url),
        safe = html_escape(&link.safe_page_url),
        ac = html_escape(&link.allowed_countries.join(",")),
        bc = html_escape(&link.blocked_countries.join(",")),
        bi = html_escape(&link.blocked_ips.join("\n")),
        bis = html_escape(&link.blocked_isps.join("\n")),
        mc = link.max_clicks,
        pt = link.param_ttl,
        ah = html_escape(&link.allowed_hours),
    )
}

// ============================
// LOGS PAGE
// ============================
pub fn logs_page(logs_b64: &str) -> String {
    let extra_css = ".geo-info{font-size:12px;color:#b4b4b4}.ua-info{max-width:220px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px;color:#b0b0b0}.filters{display:flex;gap:8px;flex-wrap:wrap;margin-bottom:12px}.filter-btn{padding:6px 12px;border-radius:8px;border:1px solid #2a2a2a;background:#1f1f1f;color:#9aa;cursor:pointer;font-size:12px}.filter-btn.active,.filter-btn:hover{background:#667eea;color:#fff;border-color:#667eea}";
    format!(r#"{head}{sidebar}
<div class="main">
<h1 style="margin-bottom:16px">Logs de Acesso</h1>
<div class="filters">
<button class="filter-btn active" onclick="filterLogs('all',this)">Todos</button>
<button class="filter-btn" onclick="filterLogs('allowed',this)">Permitidos</button>
<button class="filter-btn" onclick="filterLogs('blocked',this)">Bloqueados</button>
<form method="POST" action="/m4ciel7/logs/clear" onsubmit="return confirm('Apagar todos os logs?');"><button class="btn danger" type="submit">Apagar logs</button></form>
</div>
<div class="card"><table class="table"><thead><tr><th>Data/Hora</th><th>IP</th><th>Pais</th><th>ISP</th><th>Status</th><th>Motivo</th><th>User Agent</th></tr></thead><tbody id="logs-table"></tbody></table></div></div>
<script>
const allLogs=JSON.parse(atob('{b64}'));let currentFilter='all';
function renderLogs(logs){{const t=document.getElementById('logs-table');const f=document.createDocumentFragment();logs.forEach(l=>{{const tr=document.createElement('tr');const d=new Date(l.timestamp);const ds=isNaN(d.getTime())?'-':d.toLocaleString('pt-BR');function td(v){{const e=document.createElement('td');e.textContent=v;return e;}}tr.appendChild(td(ds));tr.appendChild(td(l.ip||'-'));const tc=document.createElement('td');const sc=document.createElement('span');sc.className='geo-info';sc.textContent=l.country||l.country_code||'-';tc.appendChild(sc);tr.appendChild(tc);const ti=document.createElement('td');const si=document.createElement('span');si.className='geo-info';si.textContent=l.isp||'-';ti.appendChild(si);if(l.is_vpn){{const v=document.createElement('span');v.className='badge badge-warning';v.style.marginLeft='6px';v.textContent='VPN';ti.appendChild(v);}}tr.appendChild(ti);const ts=document.createElement('td');const sb=document.createElement('span');sb.className=l.blocked?'badge badge-danger':'badge badge-success';sb.textContent=l.blocked?'Bloqueado':'Permitido';ts.appendChild(sb);tr.appendChild(ts);tr.appendChild(td(l.reason||'-'));const tu=document.createElement('td');const su=document.createElement('span');su.className='ua-info';const ua=l.user_agent||'';su.title=ua;su.textContent=ua.substring(0,60);tu.appendChild(su);tr.appendChild(tu);f.appendChild(tr);}});t.textContent='';t.appendChild(f);}}
function filterLogs(f,b){{currentFilter=f;document.querySelectorAll('.filter-btn').forEach(x=>x.classList.remove('active'));if(b)b.classList.add('active');let fl=allLogs;if(f==='allowed')fl=allLogs.filter(l=>!l.blocked);else if(f==='blocked')fl=allLogs.filter(l=>l.blocked);renderLogs(fl);}}
renderLogs(allLogs);
const es=new EventSource('/m4ciel7/logs/stream');
es.addEventListener('log',ev=>{{try{{const l=JSON.parse(ev.data);allLogs.unshift(l);if(allLogs.length>1000)allLogs.pop();filterLogs(currentFilter,document.querySelector('.filter-btn.active'));}}catch(e){{}}}});
es.addEventListener('clear',()=>{{allLogs.length=0;renderLogs([]);}});
</script></body></html>"#,
        head = head("Logs", extra_css),
        sidebar = sidebar("Logs"),
        b64 = logs_b64,
    )
}

// ============================
// CONFIG PAGE
// ============================
pub fn config_page(param_name: &str, only_fb: bool) -> String {
    let checked = if only_fb { "checked" } else { "" };
    format!(r#"{head}{sidebar}
<div class="main" style="max-width:720px">
<h1 style="margin-bottom:16px">Configuracoes</h1>
<form method="POST">
<div class="card"><h2>Parametro</h2>
<div class="form-group"><label>Nome do parametro</label><input type="text" name="param_name" value="{pn}"><div class="hint">Exemplo: ?{pn}=codigo</div></div></div>
<div class="card"><h2>Modo de Trafego</h2>
<div class="form-group"><label class="checkbox"><input type="checkbox" name="only_fb_ads" {checked}> Somente anuncios Facebook/Instagram</label>
<div class="hint">Desative para testar links fora do Facebook/Instagram.</div></div></div>
<div class="card"><div class="hint">Regras fixas: parametro secreto, mobile apenas, bloqueio de bots.</div>
<div style="margin-top:12px"><button class="btn primary" type="submit" style="padding:10px 14px;border-radius:8px">Salvar</button></div></div>
</form></div></body></html>"#,
        head = head("Configuracoes", ""),
        sidebar = sidebar("Configuracoes"),
        pn = html_escape(param_name),
        checked = checked,
    )
}

// ============================
// ERROR PAGE
// ============================
pub fn error_page(msg: &str) -> String {
    format!(r#"<!DOCTYPE html><html lang="pt-BR"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0"><title>Erro</title>
<style>body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0f0f0f;color:#fff;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0}}.box{{background:#1a1a1a;border:1px solid #333;border-radius:12px;padding:28px;max-width:520px;width:90%;text-align:center}}h1{{font-size:18px;margin:0 0 10px}}p{{color:#bbb;font-size:14px;margin:0 0 18px}}a{{color:#667eea;text-decoration:none}}</style></head><body>
<div class="box"><h1>Erro</h1><p>{}</p><a href="javascript:history.back()">Voltar</a></div></body></html>"#, html_escape(msg))
}
