import sys
import pandas as pd
import matplotlib.pyplot as plt
from matplotlib import cm
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parents[1]
RESULTS_DIR = BASE_DIR / "benchmark_results" / "html_reports"
PLOTS_DIR = RESULTS_DIR
PLOTS_DIR.mkdir(parents=True, exist_ok=True)

plt.rcParams.update({
    "figure.dpi": 120,
    "savefig.dpi": 300,
    "font.size": 11,
    "axes.titlesize": 14,
    "axes.labelsize": 12,
    "xtick.labelsize": 10,
    "ytick.labelsize": 10,
    "axes.grid": True,
    "grid.linestyle": ":",
    "grid.alpha": 0.5,
})

def read_first_table(p: Path) -> pd.DataFrame:
    with open(p, "r", encoding="utf-8") as f:
        return pd.read_html(f, flavor="lxml")[0]

def upsert_block(html_path: Path, block_id: str, html_block: str):
    s = html_path.read_text(encoding="utf-8")
    start_tag = f"<!-- PLOT_BLOCK_START:{block_id} -->"
    end_tag = f"<!-- PLOT_BLOCK_END:{block_id} -->"
    if start_tag in s and end_tag in s:
        start = s.index(start_tag)
        end = s.index(end_tag) + len(end_tag)
        s = s[:start] + start_tag + "\n" + html_block + "\n" + end_tag + s[end:]
        html_path.write_text(s, encoding="utf-8")
        return
    i = s.lower().rfind("</table>")
    if i == -1:
        return
    injected = s[: i + len("</table>")] + "\n" + start_tag + "\n" + html_block + "\n" + end_tag + "\n" + s[i + len("</table>") :]
    html_path.write_text(injected, encoding="utf-8")

ing_html = RESULTS_DIR / "ingestion-service_container_performance.html"
idx_html = RESULTS_DIR / "indexing-service_container_performance.html"
sea_html = RESULTS_DIR / "search-service_container_performance.html"
wf_html  = RESULTS_DIR / "system_workflow_performance.html"

missing = [p for p in [ing_html, idx_html, sea_html, wf_html] if not p.exists()]
if missing:
    print("Brak plików:", ", ".join(str(x) for x in missing)); sys.exit(1)

def latency_plot(html_file: Path, name: str):
    df = read_first_table(html_file)
    df["Avg Response Time (ms)"] = pd.to_numeric(df["Avg Response Time (ms)"], errors="coerce")
    fig, ax = plt.subplots(figsize=(8, 5))
    endpoints = df["Endpoint"].astype(str).tolist()
    values = df["Avg Response Time (ms)"].tolist()
    colors = [cm.tab10(i % 10) for i in range(len(endpoints))]
    bars = ax.bar(endpoints, values, color=colors, edgecolor="black", linewidth=0.6)
    ymax = max(values) if values else 0
    ax.set_ylim(0, ymax * 1.15 if ymax > 0 else 1)
    ax.bar_label(bars, fmt="%.0f", padding=3)
    ax.set_title(f"{name.capitalize()} Service – Average Response Time")
    ax.set_xlabel("Endpoint")
    ax.set_ylabel("Time (ms)")
    for tick in ax.get_xticklabels():
        tick.set_rotation(30)
        tick.set_ha("right")
    fig.tight_layout()
    img_name = f"{name}-service_latency.png"
    fig.savefig(PLOTS_DIR / img_name)
    plt.close(fig)
    html_block = f"<div><h3>Latency Plot</h3><img src='{img_name}' alt='{name} latency' style='max-width:100%;height:auto;'/></div>"
    upsert_block(html_file, f"{name.upper()}_LATENCY", html_block)

latency_plot(ing_html, "ingestion")
latency_plot(idx_html, "indexing")
latency_plot(sea_html, "search")

wf = read_first_table(wf_html)
for c in ["Total Time (ms)", "Ingest Time (ms)", "Index Time (ms)", "Search Time (ms)"]:
    wf[c] = pd.to_numeric(wf[c], errors="coerce")
wf = wf.sort_values("Book ID")
x = range(len(wf))
ing = wf["Ingest Time (ms)"]
idx = wf["Index Time (ms)"]
sea = wf["Search Time (ms)"]

fig1, ax1 = plt.subplots(figsize=(8, 5))
c_ing, c_idx, c_sea = cm.Set2(0), cm.Set2(1), cm.Set2(2)
ax1.bar(x, ing, label="Ingest", color=c_ing, edgecolor="black", linewidth=0.6)
ax1.bar(x, idx, bottom=ing, label="Index", color=c_idx, edgecolor="black", linewidth=0.6)
ax1.bar(x, sea, bottom=ing+idx, label="Search", color=c_sea, edgecolor="black", linewidth=0.6)
ax1.set_title("System Workflow Breakdown per Book")
ax1.set_xlabel("Book ID")
ax1.set_ylabel("Time (ms)")
ax1.set_xticks(list(x), wf["Book ID"])
ymax_stack = float((ing + idx + sea).max()) if len(wf) else 0
ax1.set_ylim(0, ymax_stack * 1.15 if ymax_stack > 0 else 1)
ax1.legend()
fig1.tight_layout()
wf_break = "workflow_breakdown_stacked.png"
fig1.savefig(PLOTS_DIR / wf_break)
plt.close(fig1)

components_sum = ing + idx + sea
fig2, ax2 = plt.subplots(figsize=(8, 5))
bars = ax2.bar(x, components_sum, label="Sum(Ingest+Index+Search)", color=cm.Set3(3), edgecolor="black", linewidth=0.6)
ax2.plot(x, wf["Total Time (ms)"], marker="o", linestyle="--", label="Total Time (ms)", color="black", linewidth=1.2)
for rect, val in zip(bars, components_sum):
    ax2.text(rect.get_x() + rect.get_width()/2, rect.get_height() + (ymax_stack*0.02 if ymax_stack>0 else 0.5),
             f"{int(val)}", ha="center", va="bottom", fontsize=9)
diff = wf["Total Time (ms)"] - components_sum
for i, d in enumerate(diff):
    if abs(d) > 0:
        ax2.text(i, max(components_sum.iloc[i], wf["Total Time (ms)"].iloc[i]) * 1.03, f"Δ={int(d)}", ha="center", fontsize=9)
ax2.set_title("Total vs Components Sum (Validation)")
ax2.set_xlabel("Book ID")
ax2.set_ylabel("Time (ms)")
ax2.set_xticks(list(x), wf["Book ID"])
ax2.legend()
fig2.tight_layout()
wf_total = "workflow_total_vs_components.png"
fig2.savefig(PLOTS_DIR / wf_total)
plt.close(fig2)

wf_block = (
    f"<div><h3>Workflow Plots</h3>"
    f"<div><img src='{wf_break}' alt='workflow breakdown' style='max-width:100%;height:auto;'/></div>"
    f"<div style='margin-top:12px;'><img src='{wf_total}' alt='workflow total vs sum' style='max-width:100%;height:auto;'/></div>"
    f"</div>"
)
upsert_block(wf_html, "WORKFLOW_PLOTS", wf_block)

print("Plots are ready and saved in png")
