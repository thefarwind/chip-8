pipeline {
  agent {
    docker {
      image 'rust:1.46'
    }

  }
  stages {
    stage('Build') {
      steps {
        sh 'cargo check --message-format=json'
      }
      post {
        always {
          recordIssues tool: Cargo
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
