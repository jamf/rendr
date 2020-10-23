docs/build/index.html:
	cd docs && mdbook build

open:
	open docs/book/index.html

clean:
	rm -rf docs/book
