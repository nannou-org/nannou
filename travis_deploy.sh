# A script to let travis build the mdbook html and deploy it.
git config user.email "travis@travis-ci.org"
git config user.name "Travis CI"
git stash &&
mdbook build &&
git remote add nannou-org https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git
git fetch nannou-org &&
git checkout deploy &&
cp -r book/* . &&
git add -A . &&
git commit -m "Automated commit in preparation for deployment: $TRAVIS_BUILD_NUMBER" &&
git push --force --quiet nannou-org deploy
