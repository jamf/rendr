@Library(['tools', "client-apps"]) _

pipeline {
    agent none

    parameters {
        booleanParam(name: 'RELEASE', defaultValue: false, description: 'Publish artifacts to GitHub Releases')
        string(name: 'VERSION', defaultValue: '', description: 'Release version')
    }

    environment {
        VERSION = "${params.RELEASE ? params.VERSION : env.TAG_NAME}"
    }

    stages {
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
                        sh 'cargo test'
                        sh 'cargo build --release'
                        sh 'mv target/release/rendr rendr-linux'
                        archiveArtifacts 'rendr-linux'
                        stash name: 'linux-cli', includes: 'rendr-linux'
                    }
                }

                stage('macOS build') {
                    agent { label "${anka 'macos.10.15-build'}" }

                    steps {
                        sh 'brew install rust'
                        sh 'cargo test'
                        sh 'cargo build --release'
                        sh 'mv target/release/rendr rendr-darwin'
                        sh 'openssl sha256 -r rendr-darwin | awk \'{print $1}\' > rendr-darwin.sha256'
                        archiveArtifacts 'rendr-darwin,rendr-darwin.sha256'
                        stash name: 'mac-cli', includes: 'rendr-darwin'
                        stash name: 'mac-sha', includes: 'rendr-darwin.sha256'
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
                    yamlFile 'rust-pod.yaml'
                }
            }

            environment {
                GITHUB_USER = 'jamfdevops'
                GITHUB_TOKEN = credentials 'github-token'
            }

            steps {
                // Create GitHub release
                unstash 'mac-cli'
                unstash 'linux-cli'
                sh label: 'Creating GitHub release', script: "hub release create $VERSION -m $VERSION -t master -a rendr-darwin -a rendr-linux"

                // Update Jamf Homebrew tap with latest rendr version
                dir('homebrew-tap') {
                    git url: 'https://github.com/jamf/homebrew-tap', branch: 'master', changelog: false, poll: false, credentialsId: 'e06e287d-0fcb-4f24-8137-7f7f9c60e09f'
                    unstash 'mac-sha'

                    script {
                        def sha256 = readFile('rendr-darwin.sha256').trim()
                        def metadata = """
                                |version: "$env.VERSION"
                                |url: "https://github.com/jamf/rendr/releases/download/$env.VERSION/rendr-darwin"
                                |sha256: "$sha256"
                                """.trim().stripMargin()
                        sh label: 'Update Homebrew metadata', script: "echo '$metadata' > metadata/rendr.yaml"
                        sh label: 'Inspect Homebrew metadata', script: 'cat metadata/rendr.yaml'
                        sh label: 'Pushing changes to Homebrew tap', script: """
                           git config user.email "devops@jamf.com"
                           git config user.name "Jenkins"
                           hub add metadata/rendr.yaml
                           hub commit -m "Update rendr formula to version $VERSION"
                           git config --local credential.helper "!f() { echo username=\\$GITHUB_USER; echo password=\\$GITHUB_TOKEN; }; f"
                           hub push origin master
                           """
                    }
                }
            }
        }
    }
}
