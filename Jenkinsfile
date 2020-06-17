pipeline {
    agent {
        kubernetes {
            label 'rust'
            defaultContainer 'rust'
            yaml '''
                apiVersion: v1
                kind: Pod
                spec:
                  containers:
                  - name: rust
                    image: docker.jamf.build/rust:1.44.0
                    tty: true
                    command:
                    - cat
                '''
        }
    }

    stages {
        stage('Build') {
            parallel {
                stage('Linux build') {
                    steps {
                        sh 'cargo build --release'
                        sh 'mv target/release/express express-linux'
                        archiveArtifacts 'express-linux'
                    }
                }

                stage('macOS build') {
                    agent { label "${anka 'macos.10.15-build'}" }

                    steps {
                        sh 'brew install rust'
                        sh 'cargo build --release'
                        sh 'mv target/release/express express-darwin'
                        archiveArtifacts 'express-darwin'
                    }
                }
            }
        }
    }
}
