.PHONY: book
deploy: book
	@echo "====> deploying to github"
	rm -rf /tmp/rust-tutorials_book
	git worktree prune
	git worktree add /tmp/rust-tutorials_book gh-pages
	rm -rf /tmp/rust-tutorials_book/*
	cp -rp build/book/* /tmp/rust-tutorials_book/
	cd /tmp/rust-tutorials_book && \
		git add -A && \
		git commit -m "deployed on $(shell date) by ${USER}" && \
		git push origin gh-pages

book:
	mdbook build
