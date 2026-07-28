#ifndef _H_IRISOBJECTBASE_
#define _H_IRISOBJECTBASE_

#include "IrisDevHeader.h"
#include "IrisStringTag.h"
#include "IrisIntegerTag.h"

#include "IrisInterpreter/IrisNativeModules/IrisKernel.h"

class IrisObjectBase : public IIrisClass
{
public:
	static IrisValue InitializeFunction(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue GetClass(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue GetObjectID(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue ToString(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue Equal(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue NotEqual(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue LogicOr(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue LogicAnd(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue Fix(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue Unfix(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);
	static IrisValue IsFixed(const IrisValue&, IIrisValues*, IIrisValues*, IIrisContextEnvironment*, IIrisThreadInfo*);

public:

	void Mark(void* pNativeObjectPointer) {}

	const char* NativeClassNameDefine() const {
		return "Object";
	}

	IIrisClass* NativeSuperClassDefine() const {
		// Special
		return nullptr;
	}

	// Ãÿ ‚¥¶¿Ì
	void* NativeAlloc() {
		return nullptr;
	}

	void NativeFree(void* pNativePointer) {
		return;
	}

	int GetTrustteeSize(void* pNativePointer) {
		return 0;
	}

	void NativeClassDefine() {

		IrisDevUtil::AddModule(this, IrisDevUtil::GetModule("Kernel"));

		IrisDevUtil::AddInstanceMethod(this, "__format", InitializeFunction, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "to_string", ToString, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "==", Equal, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "!=", NotEqual, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "&&", LogicAnd, 1, false);
		IrisDevUtil::AddInstanceMethod(this, "||", LogicOr, 1, false);

		IrisDevUtil::AddInstanceMethod(this, "unfix", Unfix, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "fix", Fix, 0, false);
		IrisDevUtil::AddInstanceMethod(this, "fixed", IsFixed, 0, false);

		IrisDevUtil::AddGetter(this, "@__class", GetClass);
		IrisDevUtil::AddGetter(this, "@object_id", GetObjectID);

	}

	IrisObjectBase();
	~IrisObjectBase();
};

#endif