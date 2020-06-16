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
            steps {
                sh 'cargo build --release'
                archiveArtifacts 'target/release/express'
            }
        }
    }
}
