@Library(['tools', "client-apps"]) _

def rustPod = '''
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

pipeline {
    agent none

    parameters {
        booleanParam(name: 'RELEASE', defaultValue: false, description: 'Publish artifacts to GitHub Releases')
        string(name: 'VERSION', defaultValue: '', description: 'Release version')
    }

    stages {
        stage ('Run tests') {
            agent {
                kubernetes {
                    label 'rust'
                    defaultContainer 'rust'
                    yaml rustPod
                }
            }

            steps {
                sh 'cargo test'
            }
        }

        stage('Build') {
            parallel {
                stage('Linux build') {
                    agent {
                        kubernetes {
                            label 'rust'
                            defaultContainer 'rust'
                            yaml rustPod
                        }
                    }

                    steps {
                        sh 'cargo build --release'
                        sh 'mv target/release/express express-linux'
                        archiveArtifacts 'express-linux'
                        stash name: 'linux-cli', includes: 'express-linux'
                    }
                }

                stage('macOS build') {
                    agent { label "${anka 'macos.10.15-build'}" }

                    steps {
                        sh 'brew install rust'
                        sh 'cargo build --release'
                        sh 'mv target/release/express express-darwin'
                        archiveArtifacts 'express-darwin'
                        stash name: 'mac-cli', includes: 'express-darwin'
                    }
                }
            }
        }

        stage('Release') {
            when {
                anyOf {
                    buildingTag()
                    expression { params.RELEASE }
                }
            }

            agent {
                kubernetes {
                    label 'rust'
                    defaultContainer 'github'
                    yaml rustPod
                }
            }

            environment {
                GITHUB_TOKEN = 'github-token'
                VERSION = "${params.RELEASE ? params.VERSION : env.TAG_NAME}"
            }

            steps {
                unstash 'mac-cli'
                unstash 'linux-cli'
                sh "hub release create $VERSION -t master -a express-darwin -a express-linux"
            }
        }
    }
}
