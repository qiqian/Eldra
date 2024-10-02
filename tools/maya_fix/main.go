package main

import (
	"bufio"
	"compress/gzip"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

//TIP To run your code, right-click the code and select <b>Run</b>. Alternatively, click
// the <icon src="AllIcons.Actions.Execute"/> icon in the gutter and select the <b>Run</b> menu item from here.

func main() {
	//TIP Press <shortcut actionId="ShowIntentionActions"/> when your caret is at the underlined or highlighted text
	// to see how GoLand suggests fixing it.
	dir := os.Args[1]

	libRegEx, e := regexp.Compile("^.+\\.(ass.gz)$")
	if e != nil {
		log.Fatal(e)
	}

	e = filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err == nil && libRegEx.MatchString(info.Name()) {
			cleanup_ass(path)
		}
		return nil
	})
	if e != nil {
		log.Fatal(e)
	}
}

func Readln(r *bufio.Reader) (string, error) {
	var (
		isPrefix bool  = true
		err      error = nil
		line, ln []byte
	)
	for isPrefix && err == nil {
		line, isPrefix, err = r.ReadLine()
		ln = append(ln, line...)
	}
	return string(ln), err
}

func cleanup_ass(ass string) {
	result := ""
	remain := []byte{}
	find_color_mgr := false
	{
		file, err := os.Open(ass)
		if err != nil {
			fmt.Println(err)
			return
		}
		defer file.Close()

		reader, err := gzip.NewReader(file)
		if err != nil {
			fmt.Println(err)
			return
		}
		defer reader.Close()

		scanner := bufio.NewReader(reader)

		fmt.Println("cleanup for " + ass)
		drop_color_mgr := false
		line, e := Readln(scanner)
		for e == nil {
			if strings.HasPrefix(strings.Trim(line, " "), "color_manager ") {
				// drop line
				fmt.Println(line)
				find_color_mgr = true
			} else if drop_color_mgr {
				// drop line
				fmt.Println(line)
				if strings.Trim(line, " ") == "}" {
					drop_color_mgr = false
					result += "\n"
					// read to end
					remain, e = io.ReadAll(scanner)
					// stop
					break
				}
			} else if strings.HasPrefix(line, "color_manager_syncolor") {
				// drop line
				fmt.Println(line)
				find_color_mgr = true
				drop_color_mgr = true
			} else {
				result += line + "\n"
			}

			line, e = Readln(scanner)
		}
	}

	if find_color_mgr {
		// write back
		file, err := os.Create(ass)
		if err != nil {
			fmt.Println(err)
			return
		}
		defer file.Close()
		w := gzip.NewWriter(file)
		w.Write([]byte(result))
		w.Write(remain)
		defer w.Close()
	}
}
