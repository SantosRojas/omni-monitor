#[cfg(feature = "frontend")]
use leptos::prelude::*;
use crate::models::DashboardSignal;

#[component]
pub fn DashboardChart(
    signal: DashboardSignal,
    width: f64,
    height: f64,
) -> impl IntoView {
    let values = signal.values;
    if values.is_empty() {
        return view! { <div class="chart-container"><p style="text-align:center;color:var(--text-muted);padding:40px;">{"Sin datos para ".to_string() + &signal.display_name.unwrap_or(signal.internal_name)}</p></div> }.into_any();
    }

    let padding = (60.0, 20.0, 20.0, 60.0); // top, right, bottom, left
    let chart_w = width - padding.1 - padding.3;
    let chart_h = height - padding.0 - padding.2;

    let min_val = values.iter().map(|v| v.value).fold(f64::MAX, f64::min);
    let max_val = values.iter().map(|v| v.value).fold(f64::MIN, f64::max);
    let range = (max_val - min_val).max(1.0);
    let min_ts = values.first().map(|v| v.timestamp.and_utc().timestamp() as f64).unwrap_or(0.0);
    let max_ts = values.last().map(|v| v.timestamp.and_utc().timestamp() as f64).unwrap_or(1.0);
    let ts_range = (max_ts - min_ts).max(1.0);

    let to_x = move |ts: f64| padding.3 + (ts - min_ts) / ts_range * chart_w;
    let to_y = move |val: f64| padding.0 + chart_h - (val - min_val) / range * chart_h;

    let points: Vec<String> = values.iter().map(|v| {
        let x = to_x(v.timestamp.and_utc().timestamp() as f64);
        let y = to_y(v.value);
        format!("{:.1},{:.1}", x, y)
    }).collect();
    let points_str = points.join(" ");

    let y_ticks = 5;
    let y_labels: Vec<_> = (0..=y_ticks).map(|i| {
        let val = min_val + (range * i as f64 / y_ticks as f64);
        let y = to_y(val);
        (val, y)
    }).collect();

    let x_ticks = 6;
    let x_labels: Vec<_> = (0..=x_ticks).map(|i| {
        let ts = min_ts + (ts_range * i as f64 / x_ticks as f64);
        let x = to_x(ts);
        let dt = chrono::DateTime::from_timestamp(ts as i64, 0)
            .map(|d| d.format("%m/%d %H:%M").to_string())
            .unwrap_or_default();
        (dt, ts, x)
    }).collect();

    view! {
        <div class="chart-container">
            <svg viewBox={format!("0 0 {} {}", width, height)} preserveAspectRatio="xMidYMid meet">
                <rect x="0" y="0" width={width} height={height} fill="none" />

                {y_labels.iter().map(|&(val, y)| view! {
                    <>
                        <line x1={padding.3} y1={y} x2={width - padding.1} y2={y}
                            stroke="rgba(255,255,255,0.08)" stroke-width="1" />
                        <text x={padding.3 - 8.0} y={y + 4.0} text-anchor="end"
                            fill="var(--text-muted)" font-size="11">
                            {format!("{:.1}", val)}
                        </text>
                    </>
                }).collect::<Vec<_>>()}

                {x_labels.iter().map(|(label, _ts, x)| view! {
                    <text x={*x} y={height - padding.2 + 16.0} text-anchor="middle"
                        fill="var(--text-muted)" font-size="10">
                        {label.clone()}
                    </text>
                }).collect::<Vec<_>>()}

                <polyline
                    points={points_str.clone()}
                    fill="none"
                    stroke="var(--accent)"
                    stroke-width="2"
                    stroke-linejoin="round"
                    stroke-linecap="round"
                />

                <circle
                    cx={to_x(values.last().unwrap().timestamp.and_utc().timestamp() as f64)}
                    cy={to_y(values.last().unwrap().value)}
                    r="3"
                    fill="var(--accent)"
                />
            </svg>
        </div>
    }.into_any()
}
