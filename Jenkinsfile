pipeline {
  agent {
    docker {
      image 'rust:latest'
    }
  }
  stages {
    stage('Build') {
      steps {
        sh 'rustup component add rustfmt'
        sh 'rustup component add clippy'
        sh 'cargo install cargo-tarpaulin cargo-audit'
        sh 'cargo build'
        sh 'cargo test'
        sh 'cargo tarpaulin --ignore-tests'
        sh 'cargo clippy'
        sh 'cargo audit'
        // recordIssues tool: cargo(pattern: 'cargo-clippy.log')
      }
    }
  }
}
