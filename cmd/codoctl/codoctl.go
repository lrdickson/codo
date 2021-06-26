package main

import (
	// Standard library
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"path/filepath"

	// 3rd Party
	"github.com/spf13/pflag"
	"gopkg.in/yaml.v2"

	// Internal
	"codo/internal"
)


func BuildImage(imagesFolder string, imageName string) {
	// Get the contents of the image config file
	imageFolder := filepath.Join(imagesFolder, imageName)
	imageConfigFile := filepath.Join(imageFolder, "config.yaml")
	imageConfigContent, err := ioutil.ReadFile(imageConfigFile)
	if err != nil {
		log.Printf("failed to read %v config: %v\n", imageName, err)
		return
	}

	// Parse the contents
	imageConfig := make(map[interface{}]interface{})
	err = yaml.Unmarshal(imageConfigContent, &imageConfig)
	if err != nil {
		log.Printf("failed to read %v config: %v\n", imageName, err)
		return
	}

	// Create the Dockerfile text
	baseImage, configHasBaseImage := imageConfig["base-image"]
	if ! configHasBaseImage {
		log.Printf("No base image given for %v\n", imageName)
		return
	}
	dockerfileText := fmt.Sprintf("FROM %v\n", baseImage)

	// Create the Dockerfile
	dockerfileDir := filepath.Join("/","tmp", "codo", imageName)
	err = os.MkdirAll(dockerfileDir, 0755)
	if err != nil {
		log.Printf("failed to create Dockerfile directory for %v: %v\n", imageName, err)
		return
	}
	dockerfileLocation := filepath.Join(dockerfileDir, "Dockerfile")
	err = os.WriteFile(dockerfileLocation, []byte(dockerfileText), 0666)
	if err != nil {
		log.Printf("failed to write Dockerfile for %v: %v\n", imageName, err)
		return
	}

	// Build the image
	fullImageName := internal.GetFullImageName(imageName)
	buildCommand := exec.Command("sudo", "docker", "build", "-t", fullImageName, dockerfileDir)
	err = buildCommand.Run()
	if err != nil {
		log.Printf("failed to build image for %v: %v\n", imageName, err)
		return
	}
}


func BuildAllImages() {
	// Get the config folder
	configFolder := internal.GetConfigDir()

	// Build each of the images
	imagesFolder := filepath.Join(configFolder, "images")
	imagesFolderContents, err := ioutil.ReadDir(imagesFolder)
	if err != nil {
		log.Fatalf("Unable to get contents of images folder: %v", err)
	}
	for _, content := range imagesFolderContents {
		if content.IsDir() {
			BuildImage(imagesFolder, content.Name())
		}
	}
}


func main() {
	// Parse command line arguments
	buildAllFlag := pflag.BoolP("buildall", "B", false, "Build images")
	pflag.Parse()

	// Check if images are to be built
	if *buildAllFlag {
		BuildAllImages()
	}
}
