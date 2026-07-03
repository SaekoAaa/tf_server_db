pipeline {
    agent any 

    options {
        timestamps() // Shows timestamps in Jenkins logs
        timeout(time: 30, unit: 'MINUTES') // Prevents jobs from hanging forever
        disableConcurrentBuilds() // Prevents overlapping builds from causing conflicts
    }
    parameters {
        choice(name: 'DEPLOY_ENV', choices: ['staging', 'production'], description: 'Deploy type?')
        booleanParam(name: 'DEBUG_MODE', defaultValue: false, description: 'Enable debug?')
    }
    environment {
        REGISTRY = "ghcr.io"
        IMAGE_NAME = "${REGISTRY}/saekoaaa/rust-app_2"
        IMAGE_TAG = "${env.BUILD_NUMBER}"
        WEB_URL = "https://github.com/saekoaaa/tf_server_db"
        DOCKER_BUILDKIT = 1
    }

    stages {
        // Jenkins Declarative Pipelines check out code automatically by default.
        // The explicit "Checkout Code" stage has been removed as it was redundant.

        stage('Unit Tests') {
            agent {
                docker { 
                    image 'rust:1.90-slim'
                    reuseNode true 
                    args '-v jenkins_cargo_cache:/usr/local/cargo/registry -v jenkins_cargo_git:/usr/local/cargo/git'
                }
            }
            steps {
                echo "Running tests in isolated Rust container..."
                sh 'cargo test'
            }
        }

        stage('Build & Push to GHCR') {
            steps {
                withCredentials([usernamePassword(credentialsId: 'github_jenkins_token', passwordVariable: 'GH_TOKEN', usernameVariable: 'GH_USER')]) {
                    
                    echo "Logging into GitHub Container Registry..."
                    // Single quotes are correctly used here to pass variables to Bash, which hides secrets from Jenkins logs.
                    sh 'echo "$GH_TOKEN" | docker login $REGISTRY -u "$GH_USER" --password-stdin'
                    
                    echo "Building and pushing via buildx..."
                    // Added --provenance=false to prevent untagged attestation layers in GHCR
                    sh """
                    docker buildx build --push \
                        -t ${IMAGE_NAME}:${IMAGE_TAG} \
                        -t ${IMAGE_NAME}:${params.DEPLOY_ENV}-latest \
                        --label org.opencontainers.image.source=${WEB_URL} \
                        --provenance=false \
                        .
                    """
                }
            }
        }

        stage('Deploy staging') {
            steps {
                echo "Deployed staging build"
            }
        }

        stage('Deploy production') {
            when {
                expression { params.DEPLOY_ENV == 'production' }
            }
            steps {
                input message: "Deploy production build?", ok: "Deployed"
                echo "Deploying production build ${IMAGE_NAME}:${IMAGE_TAG}"
            }
        }
    }
    
    post {
        always {
            sh "docker logout ${REGISTRY}"
            // deleteDir() // Optional: Cleans up the workspace after the build finishes
        }
        success {
            echo "✅ Pipeline completed successfully! Image pushed: ${IMAGE_NAME}:${IMAGE_TAG}"
        }
        failure {
            echo "❌ Pipeline failed! Check the logs."
        }
    }
}
