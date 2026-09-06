use crate::{Dependency, GitSource, LockedPackage, PackageError, SourceLock, files, git, lock};
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Clone, Copy, Debug, Default)]
pub struct InstallOptions {
    pub allow_local_git: bool,
}

pub fn install(root: &Path, options: InstallOptions) -> Result<SourceLock, PackageError> {
    let root = fs::canonicalize(root)?;
    let root_package = lock::snapshot(&lock::root_files(&root)?, None)?;
    for dependency in root_package.manifest.dependencies.values() {
        if dependency.git.is_local() && !options.allow_local_git {
            return Err(PackageError::LocalGitDenied);
        }
    }
    let lock_path = files::safe_path(&root, Path::new("iris.lock"))?;
    if lock_path.exists() {
        let existing = lock::read_lock(&root)?;
        if !options.allow_local_git
            && existing.packages.iter().any(|package| {
                package
                    .source
                    .as_ref()
                    .is_some_and(|source| source.git.is_local())
            })
        {
            return Err(PackageError::LocalGitDenied);
        }
        if existing.packages.last() != Some(&root_package) {
            return Err(PackageError::Integrity {
                path: root.join("iris.toml"),
            });
        }
        crate::cache::restore(&root, &existing, options.allow_local_git)?;
        return Ok(existing);
    }
    let cache = files::safe_path(&root, Path::new(".iris/sources"))?;
    fs::create_dir_all(&cache)?;
    let mut resolver = Resolver {
        root: &root,
        options,
        packages: Vec::new(),
        active: BTreeSet::new(),
    };
    resolver
        .active
        .insert(root_package.manifest.package_id.to_string());
    for dependency in root_package.manifest.dependencies.values() {
        resolver.visit(dependency)?;
    }
    resolver.packages.push(root_package);
    let result = SourceLock {
        lock_version: 1,
        packages: resolver.packages,
    };
    files::write_toml(&lock_path, &result)?;
    Ok(result)
}

struct Resolver<'root> {
    root: &'root Path,
    options: InstallOptions,
    packages: Vec<LockedPackage>,
    active: BTreeSet<String>,
}

impl Resolver<'_> {
    fn visit(&mut self, dependency: &Dependency) -> Result<(), PackageError> {
        let identity = dependency.package_id.to_string();
        if self.active.contains(&identity) {
            return Err(PackageError::Cycle(identity));
        }
        if let Some(package) = self
            .packages
            .iter()
            .find(|package| package.manifest.package_id == dependency.package_id)
        {
            return agreement(dependency, package);
        }
        let content = git::fetch(dependency, self.options.allow_local_git)?;
        let package = lock::snapshot(
            &content,
            Some(GitSource {
                git: dependency.git.clone(),
                rev: dependency.rev.clone(),
            }),
        )?;
        agreement(dependency, &package)?;
        crate::cache::publish(self.root, &package, &content)?;
        self.active.insert(identity.clone());
        for child in package.manifest.dependencies.values() {
            self.visit(child)?;
        }
        self.active.remove(&identity);
        self.packages.push(package);
        Ok(())
    }
}

pub(crate) fn agreement(
    dependency: &Dependency,
    package: &LockedPackage,
) -> Result<(), PackageError> {
    let matches_source = package
        .source
        .as_ref()
        .is_some_and(|source| source.git == dependency.git && source.rev == dependency.rev);
    if dependency.package_id != package.manifest.package_id
        || dependency.api_major != package.manifest.api_major
        || !dependency.version.matches(&package.manifest.version)
        || !matches_source
    {
        return Err(PackageError::Conflict(dependency.package_id.to_string()));
    }
    Ok(())
}

pub(crate) fn verify_lock(root: &Path, lock: &SourceLock) -> Result<(), PackageError> {
    verify_graph(lock)?;
    for package in &lock.packages {
        lock::verify_package(root, package)?;
    }
    Ok(())
}

pub(crate) fn verify_graph(lock: &SourceLock) -> Result<(), PackageError> {
    let mut identities = BTreeSet::new();
    for package in &lock.packages {
        if !identities.insert(package.manifest.package_id.clone()) {
            return Err(PackageError::Conflict(
                package.manifest.package_id.to_string(),
            ));
        }
    }
    let root_package = lock
        .packages
        .last()
        .ok_or_else(|| crate::error::invalid("lock", "empty graph"))?;
    if root_package.source.is_some()
        || lock
            .packages
            .iter()
            .filter(|package| package.source.is_none())
            .count()
            != 1
    {
        return Err(crate::error::invalid(
            "lock",
            "root must appear exactly once, last",
        ));
    }
    let mut visited = BTreeSet::new();
    let mut active = BTreeSet::new();
    let mut order = Vec::new();
    visit_locked(
        root_package,
        lock,
        &mut GraphWalk {
            visited: &mut visited,
            active: &mut active,
            order: &mut order,
        },
    )?;
    let actual: Vec<_> = lock
        .packages
        .iter()
        .map(|package| package.manifest.package_id.clone())
        .collect();
    if order != actual {
        return Err(crate::error::invalid(
            "lock",
            "unreachable or incorrectly ordered packages",
        ));
    }
    Ok(())
}

struct GraphWalk<'walk> {
    visited: &'walk mut BTreeSet<crate::PackageId>,
    active: &'walk mut BTreeSet<crate::PackageId>,
    order: &'walk mut Vec<crate::PackageId>,
}

fn visit_locked(
    package: &LockedPackage,
    lock: &SourceLock,
    walk: &mut GraphWalk<'_>,
) -> Result<(), PackageError> {
    let identity = &package.manifest.package_id;
    if walk.active.contains(identity) {
        return Err(PackageError::Cycle(identity.to_string()));
    }
    if walk.visited.contains(identity) {
        return Ok(());
    }
    walk.active.insert(identity.clone());
    for dependency in package.manifest.dependencies.values() {
        let child = lock
            .packages
            .iter()
            .find(|candidate| candidate.manifest.package_id == dependency.package_id)
            .ok_or_else(|| PackageError::Conflict(dependency.package_id.to_string()))?;
        agreement(dependency, child)?;
        visit_locked(child, lock, walk)?;
    }
    walk.active.remove(identity);
    walk.visited.insert(identity.clone());
    walk.order.push(identity.clone());
    Ok(())
}
