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
      }
    }

    stage('Test') {
      steps {
        sh 'cargo test --verbose'
      }
    }

  }
}