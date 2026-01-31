# frozen_string_literal: true
require "bundler/setup"

desc "ビルドします"
task :build do
  sh "cargo build --release"
end

desc "リリースアセットを作成します"
task :release => [:build] do
  require "zip"
  require "tomlrb"

  version = Tomlrb.load_file("./Cargo.toml")["package"]["version"]
  rm_rf "release" if Dir.exist?("release")
  mkdir "release"
  release_md = File.read("./release.md")
  File.write("./release/README.md", release_md.gsub("{{version}}", version))
  Zip::File.open("./release/vintage-#{version}.au2pkg.zip", create: true) do |zipfile|
    zipfile.mkdir("Script")
    zipfile.add("Script/vintage.auf2", "./target/release/vintage_auf2.dll")
  end
end
