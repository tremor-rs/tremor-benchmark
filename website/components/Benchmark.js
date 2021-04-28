import React, { Component } from "react";
import dynamic from "next/dynamic";
const Chart = dynamic(() => import("react-apexcharts"), {
  ssr: false,
  // loading: BenchmarkLoading,
});

class BenchmarkChart extends Component {
  constructor(props) {
    super(props);
    this.shortCommitHash = props.commitList.map((c) => c.slice(0, 6));
    this.state = {
      title: props.title,
      options: {
        chart: {
          id: "basic-bar",
          events: {
            markerClick: (c1, c2, { dataPointIndex }) =>
              window.open(
                `https://github.com/tremor-rs/tremor-runtime/commit/${props.commitList[dataPointIndex]}`
              ),
          },
        },
        yaxis: {
          title: {
            text: "MB/s",
          },
        },
        xaxis: {
          categories: this.shortCommitHash,
          tooltip: {
            enabled: false,
          },
        },
        legend: {
          show: true,
          showForSingleSeries: true,
          position: "bottom",
        },
      },
      series: [
        {
          name: props.title,
          data: props.throughputs,
        },
      ],
    };
  }

  render() {
    return (
      <div>
        <h5>{this.state.title}</h5>
        <div>
          <Chart
            options={this.state.options}
            series={this.state.series}
            type="line"
          />
        </div>
      </div>
    );
  }
}

export default BenchmarkChart;
