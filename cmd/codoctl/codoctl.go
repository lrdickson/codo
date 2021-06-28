package main

import (
	// Standard library
	"io/ioutil"
	"log"
	"path/filepath"

	// 3rd Party
	"github.com/spf13/pflag"

	// Internal
	"codo/internal"
)




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
			internal.BuildImage(imagesFolder, content.Name())
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
