pipeline {
  agent {
    docker {
      image 'rust:1.46'
    }

  }
  stages {
    stage('Build') {
      steps {
        sh 'cargo check --message-format json > cargo-check.log'
      }
      post {
        always {
          recordIssues tool: cargo(pattern: 'cargo-check.log')
        }
      }
    }
    stage('Test') {
      steps {
        sh 'cargo test --verbose'
      }
    }

  }
}
