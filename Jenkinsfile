@Library(['tools', "client-apps"]) _

pipeline {
    agent none

    stages {
        stage ('Run tests') {
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
