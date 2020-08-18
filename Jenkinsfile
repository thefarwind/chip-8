pipeline {
  agent {
    docker {
      image 'rust:1.31'
    }

  }
  stages {
    stage('Build') {
      steps {
        sh 'cargo build --verbose'
        sh 'cargo check --message-format=json'
        recordIssues tools: [cargo()]
      }
    }

    stage('Test') {
      steps {
        sh 'cargo test --verbose'
      }
    }

  }
}