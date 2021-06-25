package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"

	"github.com/spf13/pflag"
	"gopkg.in/yaml.v2"
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
	fmt.Printf("parsed config: %v\n", imageConfig)
}


func BuildAllImages(configFolder string) {
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
	buildFlag := pflag.BoolP("build", "b", false, "Build images")
	pflag.Parse()

	// Get the config folder
	home, err := os.UserHomeDir()
	if err != nil {
		log.Fatalf("Unable to get home directory: %v", err)
	}
	configFolder := filepath.Join(home, ".config", "codo")

	// Check if images are to be built
	fmt.Println(*buildFlag)
	if *buildFlag {
		BuildAllImages(configFolder)
	}
}
