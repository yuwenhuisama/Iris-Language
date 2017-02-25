#ifndef _H_IRISKERNEL_
#define _H_IRISKERNEL_

#include "IrisDevHeader.h"
#include "IrisInterpreter/IrisNativeClasses/IrisStringTag.h"
#include "IrisInterpreter/IrisNativeClasses/IrisUniqueStringTag.h"
#include "IrisCompiler.h"
#include "IrisThread/IrisThreadManager.h"
#include "IrisInterpreter.h"
#include "IrisInterpreter/IrisNativeModules/IrisGC.h"

#include <iostream>
#include <mutex>
using namespace std;

class IrisKernel : public IIrisModule
{

private:
	static recursive_mutex sm_rmEvalMutex;

public:
	static IrisValue Print(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue Require(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue Import(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue Eval(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue SRand(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue Rand(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);

public:
	IrisKernel();
	~IrisKernel();

private:

	const char* NativeModuleNameDefine() const {
		return "Kernel";
	}

	IIrisModule* NativeUpperModuleDefine() const {
		return nullptr;
	}

	void NativeModuleDefine() {

		IrisDevUtil::AddClassMethod(this, "print", Print, 0, true);
		IrisDevUtil::AddInstanceMethod(this, "print", Print, 0, true);
		
		IrisDevUtil::AddClassMethod(this, "eval", Eval, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "eval", Eval, 1, false);

		IrisDevUtil::AddClassMethod(this, "require", Require, 0, true);
		IrisDevUtil::AddClassMethod(this, "import", Import, 0, true);

		IrisDevUtil::AddClassMethod(this, "srand", SRand, 0, true);
		IrisDevUtil::AddInstanceMethod(this, "srand", SRand, 0, true);

		IrisDevUtil::AddClassMethod(this, "rand", Rand, 2, false);
		IrisDevUtil::AddInstanceMethod(this, "rand", Rand, 2, false);
	};

};

#endif