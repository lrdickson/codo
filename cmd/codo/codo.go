package main

import (
	// Standard library
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"path/filepath"

	// 3rd party libraries
	"gopkg.in/yaml.v2"

	// Internal
	"codo/internal"
)

func main() {
	// Get the config folder
	configFolder := internal.GetConfigDir()

	// Get the contents of the image config file
	codoConfigFile := filepath.Join(configFolder, "config.yaml")
	codoConfigContent, err := ioutil.ReadFile(codoConfigFile)
	if err != nil {
		log.Fatalf("failed to read config: %v\n", err)
	}

	// Parse the contents
	codoConfig := make(map[interface{}]interface{})
	err = yaml.Unmarshal(codoConfigContent, &codoConfig)
	if err != nil {
		log.Fatalf("failed to parse config: %v\n", err)
	}

	// Get the default image name
	defaultImage, configHasDefaultImage := codoConfig["default-image"]
	if ! configHasDefaultImage {
		log.Fatalf("no default image found in %v\n", configFolder)
	}
	defaultImageName := defaultImage.(string)
	imageName := defaultImageName

	// Determine if attach pwd
	imageConfig := internal.GetImageConfig(imageName)
	attachPwd := imageConfig["attach-pwd"].(bool)

	// Run the command in the docker container
	fullImageName := internal.GetFullImageName(imageName)
	commandContents := []string{"sudo", "docker", "run", "-ti", "--rm"}
	if attachPwd {
		commandContents = append(commandContents, "-v", "\"$(pwd)\":/codo")
	}
	commandContents = append(commandContents, fullImageName)
	commandContents = append(commandContents, os.Args[1:]...)
	log.Printf("Running %v\n", commandContents)
	command := exec.Command(commandContents[0], commandContents[1:]...)
	command.Stdin = os.Stdin
	command.Stdout = os.Stdout
	command.Stderr = os.Stderr
	err = command.Run()
	if err != nil {
		log.Fatalf("failed to run command %v: %v\n", commandContents, err)
	}
}
