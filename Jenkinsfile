@Library(['tools', "client-apps"]) _

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
                    yamlFile 'rust-pod.yaml'
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
                            yamlFile 'rust-pod.yaml'
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
            // TODO
            // when {
            //     anyOf {
            //         buildingTag()
            //         expression { params.RELEASE }
            //     }
            // }

            agent {
                kubernetes {
                    label 'rust'
                    defaultContainer 'github'
                    yamlFile 'rust-pod.yaml'
                }
            }

            environment {
                TAG_NAME = "0.1.0-alpha-${env.BUILD_NUMBER}"
                GITHUB_USER = 'jamfdevops'
                GITHUB_TOKEN = credentials 'github-token'
                HUB_VERBOSE = '1'
                VERSION = "${params.RELEASE ? params.VERSION : env.TAG_NAME}"
            }

            steps {
                unstash 'mac-cli'
                unstash 'linux-cli'
                sh "hub release create $VERSION -m $VERSION -t master -a express-darwin -a express-linux"
            }
        }
    }
}
