@Library(['tools', "client-apps"]) _

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
                  - name: github
                    image: docker.jamf.build/ci/hub:2.14.2
                    tty: true
                    command:
                    - cat
            '''
        }
    }

    stages {
        stage ('Run tests') {
            steps {
                sh 'cargo test'
            }
        }

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
                        stash name: 'mac', includes: 'express-darwin'
                    }
                }
            }
        }

        stage('Release') {
            when { buildingTag() }

            environment {
                // TODO: GITHUB_TOKEN = credentials '...'
            }

            steps {
                container('github') {
                    unstash 'mac'
                    sh "hub release create $TAG_NAME -t master -a express-darwin -a express-linux"
                }
            }
        }
    }
}
