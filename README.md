# Rust App DevOps Sandbox

## OVERVIEW
This repository serves as a practical environment for implementing and studying DevOps and Site Reliability Engineering (SRE) practices using a minimal Rust web application. 

## REPOSITORY STRUCTURE
The infrastructure and application code are organized into the following components:

- **`src/` & `Cargo.toml`**: The core Rust application utilizing `axum`, `tokio`, and `sqlx` (MySQL). It includes OpenTelemetry and Pyroscope integrations for metrics and profiling.
- **`Dockerfile`**: A multi-stage Dockerfile that builds the Rust binary and packages it into a minimal Alpine-based container.
- **`compose.yaml`**: The local development and observability stack. It provisions the application along with MySQL, Cadvisor, Grafana Alloy, Prometheus, Grafana, Pyroscope, and k6.
- **`config/`**: Configuration files for the observability stack (`alloy.hcl`, `prometheus.yaml`, `otel_collector.yaml`) and Grafana dashboards.
- **`k6/`**: Load testing scripts (e.g., `test.js`, `stress.js`) executed via the `k6` container.
- **`terraform/`**: Infrastructure as Code (IaC) configuration for provisioning resources (includes `main.tf`, `nginx.conf.tpl`, etc.).
- **`Jenkinsfile`**: CI/CD pipeline definition for building and pushing the Docker image to GitHub Container Registry (GHCR).
- **`Taskfile.yaml`**: Command execution definitions (using `go-task`) for simplified local operations (building Docker images, managing database migrations, running tests).

## CURRENT STATUS & CLEANUP ACTIONS PERFORMED
The following cleanup operations have been executed on the repository to enforce better standards:
- **State Files**: Removed `terraform.tfstate` from the root directory. State files must remain strictly within the `terraform/` directory or a remote backend.
- **Version Control**: Removed `Cargo.lock` from `.gitignore`. For binary applications, `Cargo.lock` MUST be committed to ensure reproducible builds.
- **Dangling Files**: Relocated `grafana_dashboard.json` into the `config/` directory to centralize configurations.

## TODO & NEXT STEPS (LEARNING ROADMAP)
To further advance DevOps and SRE expertise, implement the following directives:

1. **Remote Terraform State**: Migrate the local `terraform.tfstate` to a remote backend (e.g., AWS S3 + DynamoDB or HashiCorp Consul) to practice state locking and collaboration.
2. **Kubernetes Deployment**: Write Helm charts or plain Kubernetes manifests (`Deployment`, `Service`, `Ingress`) to transition the application from `docker-compose` to a k8s environment (e.g., Minikube or kind).
3. **Automated Infrastructure Provisioning**: Integrate Terraform execution within a CI/CD pipeline (Jenkins or GitHub Actions) using `terraform plan` on PRs and `terraform apply` on merge.
4. **Enhanced Observability**: Implement log aggregation (e.g., Loki or Elasticsearch) alongside the existing metrics (Prometheus) and traces (OpenTelemetry) to complete the observability triad.
5. **Secrets Management**: Remove `.env` dependencies and integrate a secrets management tool (e.g., HashiCorp Vault or AWS Secrets Manager) for database credentials.
6. **Alerting**: Configure Prometheus Alertmanager to fire alerts (via Slack or email) when k6 load tests trigger high latency or error rates.
7. **Multi-Architecture Builds**: Update the `Jenkinsfile` and `Dockerfile` to build and push multi-architecture images (amd64 and arm64) using Docker Buildx.
