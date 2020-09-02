pipeline {
  agent {
    dockerfile {
      filename 'build.Dockerfile'
    }

  }
  stages {
    stage('Build') {
      steps {
        sh 'cargo build'
        sh 'cargo clippy --message-format json > cargo-clippy.log'
        sh 'cargo audit'
        recordIssues tool: cargo(pattern: 'cargo-clippy.log')
      }
    }
    stage('Test') {
      steps {
        sh 'cargo test --verbose'
        sh 'cargo tarpaulin --ignore-tests'
      }
    }
  }
}
