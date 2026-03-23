use crate::scan::format_size;

/// Generate an interactive HTML treemap using ECharts.
///
/// The generated HTML is self-contained (loads ECharts from CDN) with:
/// - Dark theme styling
/// - Breadcrumb navigation for drill-down
/// - `leafDepth: 3` — shows 3 levels at a time
/// - Tooltips with full path and formatted size
/// - Hierarchical borders (thicker at top levels)
/// - Responsive resize
pub fn generate_html(
    tree_json: &str,
    total_size: u64,
    file_count: usize,
    scan_path: &str,
    leaf_depth: u16,
) -> String {
    let total_size_str = format_size(total_size);
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>dumap — {scan_path}</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #1a1a2e; color: #e0e0e0; }}
  #header {{ padding: 12px 24px; background: #16213e; border-bottom: 1px solid #0f3460; display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px; }}
  #header h1 {{ font-size: 18px; font-weight: 600; }}
  #header .stats {{ font-size: 14px; color: #a0a0a0; }}
  #legend {{ display: flex; gap: 12px; flex-wrap: wrap; align-items: center; }}
  #legend .item {{ display: flex; align-items: center; gap: 4px; font-size: 11px; color: #b0b0b0; }}
  #legend .swatch {{ width: 10px; height: 10px; border-radius: 2px; display: inline-block; }}
  #chart {{ width: 100%; height: calc(100vh - 80px); }}
</style>
</head>
<body>
<div id="header">
  <div style="display:flex;align-items:center;gap:24px;">
    <h1>dumap — {scan_path}</h1>
    <div id="legend">
      <span class="item"><span class="swatch" style="background:#569cd6"></span>Code</span>
      <span class="item"><span class="swatch" style="background:#ce9134"></span>Image</span>
      <span class="item"><span class="swatch" style="background:#d65656"></span>Video</span>
      <span class="item"><span class="swatch" style="background:#9c56d6"></span>Audio</span>
      <span class="item"><span class="swatch" style="background:#56d69c"></span>Archive</span>
      <span class="item"><span class="swatch" style="background:#d6ce56"></span>Document</span>
      <span class="item"><span class="swatch" style="background:#d68256"></span>Database</span>
      <span class="item"><span class="swatch" style="background:#b45656"></span>Executable</span>
      <span class="item"><span class="swatch" style="background:#78b478"></span>Config</span>
      <span class="item"><span class="swatch" style="background:#5688d6"></span>Data</span>
      <span class="item"><span class="swatch" style="background:#646478"></span>Other</span>
    </div>
  </div>
  <div class="stats">{file_count} files &middot; {total_size_str} total</div>
</div>
<div id="chart"></div>
<script src="https://cdn.jsdelivr.net/npm/echarts@5/dist/echarts.min.js"></script>
<script>
var chart = echarts.init(document.getElementById('chart'));
var data = {tree_json};

function formatBytes(b) {{
  if (b >= 1099511627776) return (b/1099511627776).toFixed(1)+' TB';
  if (b >= 1073741824) return (b/1073741824).toFixed(1)+' GB';
  if (b >= 1048576) return (b/1048576).toFixed(1)+' MB';
  if (b >= 1024) return (b/1024).toFixed(1)+' KB';
  return b+' B';
}}

chart.setOption({{
  tooltip: {{
    formatter: function(info) {{
      var val = info.value;
      var path = info.treePathInfo.map(function(n){{ return n.name; }}).join('/');
      return '<b>' + echarts.format.encodeHTML(path) + '</b><br/>' + formatBytes(val);
    }}
  }},
  series: [{{
    type: 'treemap',
    data: data,
    leafDepth: {leaf_depth},
    roam: false,
    breadcrumb: {{
      top: 4,
      left: 10,
      itemStyle: {{ color: '#16213e', borderColor: '#0f3460' }},
      textStyle: {{ color: '#e0e0e0', fontSize: 13 }}
    }},
    label: {{
      show: true,
      formatter: function(p) {{
        return p.name + '\n' + formatBytes(p.value);
      }},
      fontSize: 12,
      color: '#fff'
    }},
    upperLabel: {{
      show: true,
      height: 24,
      color: '#fff',
      fontSize: 13,
      backgroundColor: 'transparent'
    }},
    itemStyle: {{
      borderColor: '#1a1a2e',
      borderWidth: 2,
      gapWidth: 1
    }},
    levels: [
      {{ itemStyle: {{ borderColor: '#555', borderWidth: 4, gapWidth: 4 }}, upperLabel: {{ show: false }} }},
      {{ itemStyle: {{ borderColor: '#444', borderWidth: 2, gapWidth: 2 }}, upperLabel: {{ show: true }} }},
      {{ itemStyle: {{ borderColor: '#333', borderWidth: 1, gapWidth: 1 }}, upperLabel: {{ show: true }} }},
      {{ colorSaturation: [0.4, 0.8], itemStyle: {{ borderColor: '#2a2a3e', borderWidth: 1, gapWidth: 1 }} }}
    ]
  }}]
}});

window.addEventListener('resize', function() {{ chart.resize(); }});
</script>
</body>
</html>"##
    )
}

#[cfg(test)]
#[path = "html_tests.rs"]
mod html_tests;
