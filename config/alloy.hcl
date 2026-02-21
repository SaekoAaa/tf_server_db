otelcol.receiver.otlp "default" {
  http {
    endpoint = "0.0.0.0:4318"
  }

  output {
    metrics = [otelcol.processor.batch.default.input]
  }
}

otelcol.processor.batch "default" {
  output {
    metrics = [otelcol.exporter.prometheus.default.input]
  }
}
otelcol.exporter.prometheus "default" {
  forward_to = [prometheus.remote_write.metrics_service.receiver]
}
prometheus.remote_write "metrics_service" {
    endpoint {
        url = "http://prom:9090/api/v1/write"

        // basic_auth {
        //   username = "admin"
        //   password = "admin"
        // }
    }
}
